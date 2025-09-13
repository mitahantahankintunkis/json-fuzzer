package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

type login_form struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

func handler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST method is allowed", http.StatusMethodNotAllowed)
		return
	}

	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read body", http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	// var data map[string]any
	var form_data login_form
	if err := json.Unmarshal(body, &form_data); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	// Naive user validation for the hypothetical scenario
	if form_data.Username != "test" {
		http.Error(w, "User does not exist", http.StatusForbidden)
		return
	}

	if form_data.Password != "1234" {
		http.Error(w, "Incorrect password", http.StatusForbidden)
		return
	}

	fmt.Fprintf(w, "Logged in as %v", form_data.Username)
}

func main() {
	http.HandleFunc("/api/login", handler)
	fmt.Println("API listening on http://api-v1:8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Println("Server error:", err)
	}
}
