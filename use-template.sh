#!/bin/bash

cd generic
for i in {3..8}
do
	mkdir -p size$i
	sed s/_SIZE/$i/g Cargo.template.toml > size$i/Cargo.toml
done
