#!/bin/bash

for i in dist/*.d.ts; do
  if [ $i = "dist/*.d.ts" ]; then
    break
  fi

  new_file_path=$(echo $i | sed 's/\(.*\.\)ts/\1cts/')

  cp $i $new_file_path
done
