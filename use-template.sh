#!/bin/bash

cd generic
for i in {3..8}
do
    mkdir -p crates/size$i
    sed s/_SIZE/$i/g Cargo.template.toml > crates/size$i/Cargo.toml
done
