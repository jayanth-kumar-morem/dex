package main

import (
    "log"
    "github.com/yourusername/dex-platform/internal/app"
)

func main() {
    if err := app.Run(); err != nil {
        log.Fatal(err)
    }
}
