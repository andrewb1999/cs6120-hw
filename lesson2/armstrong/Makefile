.PHONY: run
run: build/armstrong.json
	- cd build && brili 663 < armstrong.json
	- cd build && brili 407 < armstrong.json
	- cd build && brili 1634 < armstrong.json
	- cd build && brili 1234 < armstrong.json

build/armstrong.json: armstrong.bril build/
	cd build && bril2json < ../armstrong.bril > armstrong.json

build/:
	mkdir -p build
