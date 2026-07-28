#!/bin/sh

bin="./target/release/i2hex4wasm_example"

export ENV_WAT_PATH="./example.wat"

input(){
  jq -c -n '[
    {helo:"wrld",  id:42, f: 42.195},
    {helo:"world", id:43, f: 2.99792458}
  ]' |
    jq -c '.[]'
}

input |
  "${bin}" |
  jq -c
