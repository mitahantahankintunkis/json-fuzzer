package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	_ "github.com/mattn/go-sqlite3"
	"io"
	"net/http"
	"os"
)

type user struct {
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
	var user user
	if err := json.Unmarshal(body, &user); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	db, err := sql.Open("sqlite3", "./sqlite.db")
	if err != nil {
		http.Error(w, "Internal error", http.StatusInternalServerError)
		return
	}
	defer db.Close()

	row := db.QueryRow("SELECT name FROM users WHERE name = '" + user.Username + "' AND password = '" + user.Password + "' LIMIT 1")
	var username string
	err = row.Scan(&username)
	if err != nil {
		http.Error(w, "Login error", http.StatusUnauthorized)
		return
	}

	fmt.Fprintf(w, "Logged in as %v", username)
}

func main() {
	os.Remove("./sqlite.db")
	db, err := sql.Open("sqlite3", "./sqlite.db")

	if err != nil {
		fmt.Println("Could not open sqlite: ", err)
		return
	}

	defer db.Close()

	create := `
	CREATE TABLE users (name TEXT NOT NULL PRIMARY KEY, password TEXT NOT NULL);
	INSERT INTO users VALUES("admin", "supersecret");
	`
	_, err = db.Exec(create)
	if err != nil {
		fmt.Println("Could not init db: ", err)
		return
	}

	http.HandleFunc("/api/login", handler)
	fmt.Println("Server listening on http://backend:8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Println("Server error:", err)
	}
}
