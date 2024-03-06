#!/bin/bash

echoerr() { echo "$@" 1>&2; }

for i in {2..10}
do
    echo "output: $i"

    echoerr hello world

    sleep 1
done