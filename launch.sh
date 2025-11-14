#!/bin/bash
# Exodia Contagion Macro Launcher for Linux
# This script will check for Python, install dependencies, and run the macro

# Get the directory where the script is located
cd "$(dirname "$0")"

echo "========================================"
echo "Exodia Contagion Macro Launcher"
echo "========================================"
echo ""

# Check if Python 3 is installed
if ! command -v python3 &> /dev/null; then
    echo "ERROR: Python 3 is not installed"
    echo "Please install Python 3.6+ from your package manager:"
    echo "  Ubuntu/Debian: sudo apt install python3 python3-pip python3-venv"
    echo "  Arch: sudo pacman -S python python-pip"
    echo "  Fedora: sudo dnf install python3 python3-pip"
    exit 1
fi

echo "[1/3] Python found!"
python3 --version

# Check if pip is available
if ! python3 -m pip --version &> /dev/null; then
    echo "ERROR: pip is not available"
    echo "Please install pip:"
    echo "  Ubuntu/Debian: sudo apt install python3-pip"
    echo "  Arch: sudo pacman -S python-pip"
    exit 1
fi

echo "[2/3] Checking dependencies..."

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
    if [ $? -ne 0 ]; then
        echo "ERROR: Failed to create virtual environment"
        exit 1
    fi
fi

# Activate virtual environment and install dependencies
source venv/bin/activate
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to activate virtual environment"
    exit 1
fi

echo "Installing/updating dependencies..."
python3 -m pip install --quiet --upgrade pip
python3 -m pip install --quiet -r requirements.txt
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to install dependencies"
    exit 1
fi

echo "[3/3] Dependencies installed!"
echo ""
echo "========================================"
echo "Starting macro..."
echo "Press Ctrl+C to exit"
echo "========================================"
echo ""

# Run the macro
python3 pt-macro.py

# Exit with the same code as the macro
exit $?

