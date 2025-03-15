# Define source, output directory, and filenames
RUST_SRC = Parser.rs
OUT_DIR = bin
RUST_OUT = $(OUT_DIR)/parser.exe

# Define the Python script name
PYTHON_SCRIPT = ../tests/LexerTests.py

all: build run

build:
	rustc $(RUST_SRC) -o $(RUST_OUT)

run:
	python.exe $(PYTHON_SCRIPT)

clean:
	rm -rf $(OUT_DIR)