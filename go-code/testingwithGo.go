package main

/*
#cgo LDFLAGS: -L${SRCDIR}/../target/release -ldji_log_parser
#cgo CFLAGS: -I${SRCDIR}/../dji-log-parser/include
#include "dji_log_parser.h"
#include <stdlib.h>
*/
import "C"

import (
    "encoding/json"
    "fmt"
    "os"
    "path/filepath"
    "unsafe"
	"math"
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
    // Add other properties as needed
}

func main() {
    if len(os.Args) < 3 {
        fmt.Println("Usage: go run testingwithGo.go <input_file> <api_key>")
        os.Exit(1)
    }

    inputFile := filepath.Join(".", os.Args[1])
    apiKey := os.Args[2]

    // Check if the file exists
    fileInfo, err := os.Stat(inputFile)
    if os.IsNotExist(err) {
        fmt.Printf("Error: The file %s does not exist.\n", inputFile)
        os.Exit(1)
    }

    // Check if the file is empty
    if fileInfo.Size() == 0 {
        fmt.Printf("Error: The file %s is empty.\n", inputFile)
        os.Exit(1)
    }

    fmt.Printf("Input file: %s (Size: %d bytes)\n", inputFile, fileInfo.Size())

    // Convert inputFile and apiKey to C strings
    cInputFile := C.CString(inputFile)
    cApiKey := C.CString(apiKey)
    defer C.free(unsafe.Pointer(cInputFile))
    defer C.free(unsafe.Pointer(cApiKey))

    // Call the Rust function
    result := C.parse_dji_log(cInputFile, cApiKey)
    fmt.Printf("C.parse_dji_log returned: %v\n", bool(result))
    if !bool(result) {
        errPtr := C.get_last_error()
        errStr := C.GoString(errPtr)
        C.free_string(errPtr)
        fmt.Printf("Parsing failed: %s\n", errStr)
        os.Exit(1)
    }

    fmt.Println("Parsing successful")

    // Get the GeoJSON file path
    geojsonFilePathPtr := C.get_geojson_file_path(cInputFile)
    geojsonFilePath := C.GoString(geojsonFilePathPtr)
    C.free_string(geojsonFilePathPtr)

    // Read GeoJSON from file
    geojsonBytes, err := os.ReadFile(geojsonFilePath)
    if err != nil {
        fmt.Println("Error reading GeoJSON file:", err)
        os.Exit(1)
    }

    // Parse GeoJSON
    var geojson GeoJSON
    err = json.Unmarshal(geojsonBytes, &geojson)
    if err != nil {
        fmt.Println("Error parsing GeoJSON:", err)
        os.Exit(1)
    }

    // Print GeoJSON details
    fmt.Printf("GeoJSON Type: %s\n", geojson.Type)
    fmt.Printf("Number of Features: %d\n", len(geojson.Features))

    if len(geojson.Features) > 0 {
        firstFeature := geojson.Features[10000]
        fmt.Printf("First Feature Type: %s\n", firstFeature.Type)
        fmt.Printf("First Feature Geometry Type: %s\n", firstFeature.Geometry.Type)
        fmt.Printf("First Feature Coordinates: %v\n", firstFeature.Geometry.Coordinates)
        fmt.Printf("First Feature Time: %s\n", firstFeature.Properties.Time)
        fmt.Printf("First Feature Height: %.2f\n", firstFeature.Properties.Height)
        fmt.Printf("First Feature Speed: %.2f\n", firstFeature.Properties.Speed)
    }

    // Calculate some statistics
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
        // Calculate distance between consecutive points
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
    fmt.Printf("Max Height: %.2f\n", maxHeight)
    fmt.Printf("Total Distance: %.2f\n", totalDistance) // This will be 0 unless you implement the distance calculation
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