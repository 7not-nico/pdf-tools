#!/bin/bash

echo "PDF Tools Runner"
echo "================="
echo "Available tools:"
echo "1. pdf-opticompress"
echo "2. pdf-renamer"
echo ""

read -p "Choose tool (1 or 2): " tool

case $tool in
    1)
        echo "pdf-opticompress selected"
        echo "Commands: optimize, analyze, batch"
        read -p "Enter command: " cmd
        read -p "Enter input PDF: " input
        case $cmd in
            optimize)
                read -p "Enter output PDF: " output
                read -p "Quality (default 80): " quality
                quality=${quality:-80}
                ./pdf-opticompress/target/release/pdf-opticompress optimize "$input" "$output" --quality "$quality"
                ;;
            analyze)
                ./pdf-opticompress/target/release/pdf-opticompress analyze "$input"
                ;;
            batch)
                read -p "Enter output directory: " outdir
                read -p "Number of threads (default 4): " threads
                threads=${threads:-4}
                ./pdf-opticompress/target/release/pdf-opticompress batch "$input" --output-dir "$outdir" --threads "$threads"
                ;;
            *)
                echo "Invalid command"
                ;;
        esac
        ;;
    2)
        echo "pdf-renamer selected"
        read -p "Enter input PDF or directory: " input
        read -p "Pattern (title or filename, default title): " pattern
        pattern=${pattern:-title}
        ./pdf-renamer/target/release/pdf-renamer --input "$input" --pattern "$pattern"
        ;;
    *)
        echo "Invalid choice"
        ;;
esac