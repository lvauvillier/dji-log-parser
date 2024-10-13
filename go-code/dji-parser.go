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
	"os"
	"path/filepath"
	"unsafe"

	"github.com/peterstace/simplefeatures/geom"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Println("Usage error (two arguments only): go run dji-parser.go <input_file> <api_key>")
		os.Exit(1)
	}

	inputFile := filepath.Join(".", os.Args[1])
	apiKey := "3519165b8d4ab74ca7033a64313e6b5" //os.Args[2]

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
	fmt.Println(geojsonStr)

	var fc geom.GeoJSONFeatureCollection
	err = json.Unmarshal([]byte(geojsonStr), &fc)
	if err != nil {
		return nil, fmt.Errorf("error parsing GeoJSON: %s", err)
	}

	geometry, err := createGeometryFromFeatureCollection(fc)
	if err != nil {
		return &geom.Geometry{}, fmt.Errorf("error creating geometry from geoJson: %s", err)
	}

	return geometry, nil
}

func createGeometryFromFeatureCollection(fc geom.GeoJSONFeatureCollection) (*geom.Geometry, error) {
	var coords []float64

	for _, feature := range fc {
		if feature.Geometry.IsEmpty() {
			continue
		}

		if feature.Geometry.Type() == geom.TypePoint {
			point := feature.Geometry.MustAsPoint()
			xy, _ := point.XY()
			coords = append(coords, xy.X, xy.Y)
		}
	}

	if len(coords) < 4 {
		return nil, fmt.Errorf("not enough valid points to create a linestring")
	}

	seq := geom.NewSequence(coords, geom.DimXY)
	linestring := geom.NewLineString(seq)
	geometry := linestring.AsGeometry()
	return &geometry, nil
}
