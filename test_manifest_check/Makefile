all: target

manifest.json:
	@echo "Generating manifest..."
	@echo '{"test": "value"}' > $@

target: manifest.json
	@echo "Building target..."
	@if [ ! -f "manifest.json" ]; then \
		echo "ERROR: Manifest missing!"; \
		exit 1; \
	fi
	@echo "Target built" > $@

clean:
	rm -f target manifest.json
