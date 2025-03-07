#!/bin/bash

for i in .changeset/*.md; do
  match=$(grep -e "$1" $i)
  echo $match
  if [ -z "${match}" ]; then
    rm $i
  else
    sed -i ":a;N;s|---.*---|---\n$match\n---|;ba" $i
  fi
done