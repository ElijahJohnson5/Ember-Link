#!/bin/bash

for i in dist/cjs/*.d.ts; do
  if [ $i = "dist/cjs/*.d.ts" ]; then
    break
  fi

  new_file_path=$(echo $i | sed 's/\(.*\.\)ts/\1cts/')

  mv $i $new_file_path

  # Get just the file name
  old_file=$(echo $i | sed 's:.*/::')
  new_file=$(echo $new_file_path | sed 's:.*/::')

  map_file_path=$(echo "$i.map")

  map_file=$(echo $map_file_path | sed 's:.*/::')

  new_file_dots_escaped=$(echo $new_file | sed 's/\./\\./g')
  old_file_dots_escaped=$(echo $old_file | sed 's/\./\\./g')

  sed -i "s|\"$old_file_dots_escaped\"|\"$new_file_dots_escaped\"|" $map_file_path

  mv $map_file_path `echo $map_file_path | sed 's/\(.*\.\)ts\.map/\1cts\.map/'`
done
