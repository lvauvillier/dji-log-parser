package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
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
    AircraftName           string    `json:"aircraftName"`
    AircraftSN             string    `json:"aircraftSN"`
    Area                   string    `json:"area"`
    CameraSN               string    `json:"cameraSN"`
    CaptureNum             int       `json:"captureNum"`
    City                   string    `json:"city"`
    DetailInfoChecksum     int       `json:"detailInfoChecksum"`
    IsFavorite             int       `json:"isFavorite"`
    IsNew                  int       `json:"isNew"`
    MaxHeight              float64   `json:"maxHeight"`
    MaxHorizontalSpeed     float64   `json:"maxHorizontalSpeed"`
    MaxVerticalSpeed       float64   `json:"maxVerticalSpeed"`
    MomentPicImageBufferLen []int     `json:"momentPicImageBufferLen"`
    MomentPicLatitude       []float64 `json:"momentPicLatitude"`
    MomentPicLongitude      []float64 `json:"momentPicLongitude"`
    MomentPicShrinkImageBufferLen []int `json:"momentPicShrinkImageBufferLen"`
    NeedsUpload            int       `json:"needsUpload"`
    ProductType            string    `json:"productType"`
    RecordLineCount        int       `json:"recordLineCount"`
    StartTime              string    `json:"startTime"`
    Street                 string    `json:"street"`
    SubStreet              string    `json:"subStreet"`
    TakeOffAltitude        float64   `json:"takeOffAltitude"`
    TotalDistance          float64   `json:"totalDistance"`
    TotalTime              float64   `json:"totalTime"`
    VideoTime              int       `json:"videoTime"`
}

func main() {
	if len(os.Args) < 3 {
		fmt.Println("Usage: go run main.go <input_file> <api_key>")
		os.Exit(1)
	}

	inputFile := filepath.Join("../test-data", os.Args[1])
	apiKey := os.Args[2]

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

	// Print out some details about the parsed GeoJSON
	fmt.Printf("GeoJSON Type: %s\n", geojson.Type)
	fmt.Printf("Geometry Type: %s\n", geojson.Geometry.Type)
	fmt.Printf("Number of Coordinates: %d\n", len(geojson.Geometry.Coordinates))
	fmt.Printf("Aircraft Name: %s\n", geojson.Properties.AircraftName)
	fmt.Printf("Start Time: %s\n", geojson.Properties.StartTime)
	fmt.Printf("Max Height: %.2f\n", geojson.Properties.MaxHeight)
	fmt.Printf("Total Distance: %.2f\n", geojson.Properties.TotalDistance)
}
