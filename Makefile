ENV_FILE=.env

include $(ENV_FILE)

.PHONY: all build check-vendor

all: build

check-vendor:
	@if [ ! -d "$(ROBOCOP_VENDOR_DIR)" ]; then \
		echo "Vendor directory '$(ROBOCOP_VENDOR_DIR)' not found."; \
		if [ -n "$(ROBOCOP_REPOSITORY_URL)" ]; then \
			echo "Cloning from $(ROBOCOP_REPOSITORY_URL)..."; \
			git clone $(ROBOCOP_REPOSITORY_URL) $(ROBOCOP_VENDOR_DIR); \
		else \
			echo "Error: ROBOCOP_REPOSITORY_URL not set. Cannot clone."; \
			exit 1; \
		fi \
	fi

build-vendor:
	@if [ ! -d "$(ROBOCOP_VENDOR_DIR)" ]; then \
		echo "Vendor directory '$(ROBOCOP_VENDOR_DIR)' not found."; \
		exit 1; \
	fi
	cd $(ROBOCOP_VENDOR_DIR) && yarn && yarn make

update-vendor:
	@if [ ! -d "$(ROBOCOP_VENDOR_DIR)" ]; then \
		echo "Vendor directory '$(ROBOCOP_VENDOR_DIR)' not found."; \
		exit 1; \
	fi
	cd $(ROBOCOP_VENDOR_DIR) && git pull
	cd $(ROBOCOP_VENDOR_DIR) && yarn && yarn make

build: check-vendor build-vendor
	cargo build

run: build
	@if ! docker system info > /dev/null 2>&1; then \
		echo "Docker is not running. Starting Docker Desktop..."; \
		open -a Docker; \
		while ! docker system info > /dev/null 2>&1; do \
			echo "Waiting for Docker to start..."; \
			sleep 1; \
		done; \
		echo "Docker is now running."; \
	else \
		echo "Docker is already running."; \
	fi
	cargo run bot.conf
