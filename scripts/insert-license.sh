#!/usr/bin/env bash

# Add license header for Rust files if there is no one before.

for i in $(git ls-files --exclude-standard | grep "\.rs"); do
    # first line -> match -> print line -> quit
    matches=$(sed -n "1{/Copyright [0-9]\{4\} CeresDB Project Authors. Licensed under Apache-2.0./p;};q;" $i)
    if [ -z "${matches}" ]; then
      echo "// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0." > $i.new
      echo "" >> $i.new
      cat $i >> $i.new
      mv $i.new $i
    fi
done
