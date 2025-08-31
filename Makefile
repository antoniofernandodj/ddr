PACKAGE_NAME = ddr
VERSION      = 0.1.0
ARCH         = amd64

all: package

build:
	python3 scripts/build.py

package: build
	python3 scripts/package.py $(PACKAGE_NAME) $(VERSION) $(ARCH)

install: $(DEB_FILE)
	python3 scripts/install.py $(PACKAGE_NAME) $(VERSION) $(ARCH)

uninstall:
	python3 scripts/uninstall.py $(PACKAGE_NAME) $(VERSION) $(ARCH)

clean:
	python3 scripts/clean.py
