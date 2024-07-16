package main

/*
#cgo LDFLAGS: -L${SRCDIR}/../target/release -ldji_log_parser
#cgo CFLAGS: -I${SRCDIR}/../dji-log-parser/include
#include "dji-log-parser.h"
#include <stdlib.h>
*/
import "C"

import (
    "bufio"
    "encoding/json"
    "fmt"
    "io"
    "math"
    "os"
    "path/filepath"
    "unsafe"
	"github.com/peterstace/simplefeatures/geom"
)

type GeoJSON struct {
    Type     string    `json:"type"`
    Features []Feature `json:"features"`
}

type Feature struct {
    Type       string     `json:"type"`
    Geometry   Geometry   `json:"geometry"`
    Properties Properties `json:"properties"`
}

type Geometry struct {
    Type        string    `json:"type"`
    Coordinates []float64 `json:"coordinates"`
}

type Properties struct {
    Time   string  `json:"time"`
    Height float64 `json:"height"`
    Speed  float64 `json:"speed"`
}

func main() {
    if len(os.Args) < 3 {
        fmt.Println("Usage error (two arguments only): go run dji-parser.go <input_file> <api_key>")
        os.Exit(1)
    }

    inputFile := filepath.Join(".", os.Args[1])
    apiKey := os.Args[2]

    fileInfo, err := os.Stat(inputFile)
    if os.IsNotExist(err) {
        fmt.Printf("Error: The file %s does not exist.\n", inputFile)
        os.Exit(1)
    }

    if fileInfo.Size() == 0 {
        fmt.Printf("Error: The file %s is empty.\n", inputFile)
        os.Exit(1)
    }

    fmt.Printf("Input file: %s (Size: %d bytes)\n", inputFile, fileInfo.Size())

    file, err := os.Open(inputFile)
    if err != nil {
        fmt.Printf("Error opening file: %s\n", err)
        os.Exit(1)
    }
    defer file.Close()

    reader := bufio.NewReader(file)
    processReader(reader, apiKey)
}

func processReader(reader io.Reader, apiKey string) (*geom.Geometry, error) {
    data, err := io.ReadAll(reader)
    if err != nil {
        return nil, fmt.Errorf("error reading data: %s", err)
    }

    cData := C.CBytes(data)
    defer C.free(unsafe.Pointer(cData))
    cLength := C.size_t(len(data))
    cApiKey := C.CString(apiKey)
    defer C.free(unsafe.Pointer(cApiKey))

    geojsonPtr := C.get_geojson_string_from_bytes((*C.uchar)(unsafe.Pointer(cData)), cLength, cApiKey)
    if geojsonPtr == nil {
        errPtr := C.get_last_error()
        errStr := C.GoString(errPtr)
        C.c_api_free_string(errPtr)
        return nil, fmt.Errorf("failed to get GeoJSON: %s", errStr)
    }
    defer C.c_api_free_string(geojsonPtr)

    geojsonStr := C.GoString(geojsonPtr)

    var geojson GeoJSON
    err = json.Unmarshal([]byte(geojsonStr), &geojson)
    if err != nil {
        return nil, fmt.Errorf("error parsing GeoJSON: %s", err)
    }

    printGeoJSONDetails(geojson)
    calculateStatistics(geojson)
    return createGeometryFromData(geojson), nil
}

func printGeoJSONDetails(geojson GeoJSON) {
    fmt.Printf("GeoJSON Type: %s\n", geojson.Type)
    fmt.Printf("Number of Features: %d\n", len(geojson.Features))

    if len(geojson.Features) > 0 {
        firstFeature := geojson.Features[0]
        fmt.Printf("First Feature Type: %s\n", firstFeature.Type)
        fmt.Printf("First Feature Geometry Type: %s\n", firstFeature.Geometry.Type)
        fmt.Printf("First Feature Coordinates: %v\n", firstFeature.Geometry.Coordinates)
        fmt.Printf("First Feature Time: %s\n", firstFeature.Properties.Time)
        fmt.Printf("First Feature Height: %.2f\n", firstFeature.Properties.Height)
        fmt.Printf("First Feature Speed: %.2f\n", firstFeature.Properties.Speed)
    }
}

func calculateStatistics(geojson GeoJSON) {
    var totalDistance float64
    var maxHeight float64
    var startTime, endTime string

    for i, feature := range geojson.Features {
        if i == 0 {
            startTime = feature.Properties.Time
        }
        if i == len(geojson.Features)-1 {
            endTime = feature.Properties.Time
        }
        if feature.Properties.Height > maxHeight {
            maxHeight = feature.Properties.Height
        }
        if i > 0 {
            prevFeature := geojson.Features[i-1]
            lat1, lon1 := prevFeature.Geometry.Coordinates[1], prevFeature.Geometry.Coordinates[0]
            lat2, lon2 := feature.Geometry.Coordinates[1], feature.Geometry.Coordinates[0]
            distance := distanceHaversine(lat1, lon1, lat2, lon2)
            totalDistance += distance
        }
    }

    fmt.Printf("\nFlight Statistics:\n")
    fmt.Printf("Start Time: %s\n", startTime)
    fmt.Printf("End Time: %s\n", endTime)
    fmt.Printf("Max Height: %.2fm\n", maxHeight)
    fmt.Printf("Total Distance: %.2fkm\n", totalDistance)
}

func degreesToRadians(degrees float64) float64 {
    return degrees * math.Pi / 180
}

func distanceHaversine(lat1, lon1, lat2, lon2 float64) float64 {
    earthRadiusKm := 6371.0

    dLat := degreesToRadians(lat2 - lat1)
    dLon := degreesToRadians(lon2 - lon1)

    lat1 = degreesToRadians(lat1)
    lat2 = degreesToRadians(lat2)

    a := math.Sin(dLat/2)*math.Sin(dLat/2) +
        math.Sin(dLon/2)*math.Sin(dLon/2)*math.Cos(lat1)*math.Cos(lat2)
    c := 2 * math.Atan2(math.Sqrt(a), math.Sqrt(1-a))
    return earthRadiusKm * c
}

func createGeometryFromData(geojson GeoJSON)(*geom.Geometry) {
	numCoords := len(geojson.Features)
	coords := make([]float64, 0, numCoords)

	for _, feature := range geojson.Features {
		fmt.Println(feature)
		lng := feature.Geometry.Coordinates[0]
		lat := feature.Geometry.Coordinates[1]

		coords = append(coords, lng, lat)
	}

	sequence := geom.NewSequence(coords, geom.DimXY)
	linestring := geom.NewLineString(sequence)
	geometry := linestring.AsGeometry()
	return &geometry	
}