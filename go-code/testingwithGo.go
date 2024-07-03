package main

/*
#cgo LDFLAGS: -L${SRCDIR}/../target/aarch64-apple-darwin/release -ldji_log_parser -framework Security -framework Foundation
#include <stdlib.h>
#include <stdbool.h>

bool parse_dji_log(const char* filename, const char* api_key);
char* get_last_error();
void free_string(char* s);
char* get_geojson_file_path(const char* filename);
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"unsafe"
)

type GeoJSON struct {
	Type       string     `json:"type"`
	Geometry   Geometry   `json:"geometry"`
	Properties Properties `json:"properties"`
}

type Geometry struct {
	Type        string      `json:"type"`
	Coordinates [][]float64 `json:"coordinates"`
}

type Properties struct {
	AircraftName                  string    `json:"aircraftName"`
	AircraftSN                    string    `json:"aircraftSN"`
	Area                          string    `json:"area"`
	CameraSN                      string    `json:"cameraSN"`
	CaptureNum                    int       `json:"captureNum"`
	City                          string    `json:"city"`
	DetailInfoChecksum            int       `json:"detailInfoChecksum"`
	IsFavorite                    int       `json:"isFavorite"`
	IsNew                         int       `json:"isNew"`
	MaxHeight                     float64   `json:"maxHeight"`
	MaxHorizontalSpeed            float64   `json:"maxHorizontalSpeed"`
	MaxVerticalSpeed              float64   `json:"maxVerticalSpeed"`
	MomentPicImageBufferLen       []int     `json:"momentPicImageBufferLen"`
	MomentPicLatitude             []float64 `json:"momentPicLatitude"`
	MomentPicLongitude            []float64 `json:"momentPicLongitude"`
	MomentPicShrinkImageBufferLen []int     `json:"momentPicShrinkImageBufferLen"`
	NeedsUpload                   int       `json:"needsUpload"`
	ProductType                   string    `json:"productType"`
	RecordLineCount               int       `json:"recordLineCount"`
	StartTime                     string    `json:"startTime"`
	Street                        string    `json:"street"`
	SubStreet                     string    `json:"subStreet"`
	TakeOffAltitude               float64   `json:"takeOffAltitude"`
	TotalDistance                 float64   `json:"totalDistance"`
	TotalTime                     float64   `json:"totalTime"`
	VideoTime                     int       `json:"videoTime"`
}

func main() {
	if len(os.Args) < 3 {
		fmt.Println("Usage: go run testingwithGo.go <input_file> <api_key>")
		os.Exit(1)
	}

	inputFile := filepath.Join("../test-data", os.Args[1])
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
	if !bool(result) {
		errPtr := C.get_last_error()
		errStr := C.GoString(errPtr)
		C.free_string(errPtr)
		fmt.Printf("Parsing failed: %s\n", errStr)

		// Print more details about the input
		fmt.Printf("Input file path: %s\n", inputFile)
		fmt.Printf("Input file size: %d bytes\n", fileInfo.Size())
		fmt.Printf("API Key: %s\n", apiKey)
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
	fmt.Printf("Geometry Type: %s\n", geojson.Geometry.Type)
	fmt.Printf("Number of Coordinates: %d\n", len(geojson.Geometry.Coordinates))
	fmt.Printf("Aircraft Name: %s\n", geojson.Properties.AircraftName)
	fmt.Printf("Start Time: %s\n", geojson.Properties.StartTime)
	fmt.Printf("Max Height: %.2f\n", geojson.Properties.MaxHeight)
	fmt.Printf("Total Distance: %.2f\n", geojson.Properties.TotalDistance)
}

// Helper function to find the minimum of two integers
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

/*func main() {
	if len(os.Args) < 3 {
		fmt.Println("Usage: go run main.go <input_file> <api_key>")
		os.Exit(1)
	}

	inputFile := filepath.Join("../test-data", os.Args[1])
	apiKey := os.Args[2]

	filename := C.CString("/path/to/your/dji_log_file.txt")
    defer C.free(unsafe.Pointer(filename))

    result := C.parse_dji_log(filename)
    if bool(result) {
        fmt.Println("Parsing successful")
    } else {
        errPtr := C.get_last_error()
        errStr := C.GoString(errPtr)
        C.free_string(errPtr)
        fmt.Printf("Parsing failed: %s\n", errStr)
    }

	// Temporary file for GeoJSON output
	tmpfile, err := os.CreateTemp("", "geojson*.json")
	if err != nil {
		fmt.Println("Error creating temp file:", err)
		os.Exit(1)
	}
	defer os.Remove(tmpfile.Name())
	defer tmpfile.Close()

	// Run dji-log command
	cmd := exec.Command("../target/release/dji-log", inputFile, "--api-key", apiKey, "--geojson", tmpfile.Name())
	output, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Println("Error running dji-log:", err)
		fmt.Println("Output:", string(output))
		os.Exit(1)
	}

	// Read GeoJSON from temp file
	geojsonBytes, err := os.ReadFile(tmpfile.Name())
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
	fmt.Printf("GeoJSON Type: %s\n", geojson.Type)
	fmt.Printf("Geometry Type: %s\n", geojson.Geometry.Type)
	fmt.Printf("Number of Coordinates: %d\n", len(geojson.Geometry.Coordinates))
	fmt.Printf("Aircraft Name: %s\n", geojson.Properties.AircraftName)
	fmt.Printf("Start Time: %s\n", geojson.Properties.StartTime)
	fmt.Printf("Max Height: %.2f\n", geojson.Properties.MaxHeight)
	fmt.Printf("Total Distance: %.2f\n", geojson.Properties.TotalDistance)
}*/
