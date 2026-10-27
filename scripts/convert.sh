#!/usr/bin/env bash
set -euo pipefail

work=$(mktemp -d)
base=https://raw.githubusercontent.com/infinitered/nsfwjs/master/models/mobilenet_v2

mkdir "$work/tfjs"
curl -sSL "$base/model.json" -o "$work/tfjs/model.json"
curl -sSL "$base/group1-shard1of1" -o "$work/tfjs/group1-shard1of1"

uv venv --python 3.11 "$work/venv"
source "$work/venv/bin/activate"
uv pip install "tensorflow<2.16" "tensorflowjs<4.20" tf2onnx "onnx==1.16.2" "onnxruntime<1.20" "setuptools<70"

export TF_USE_LEGACY_KERAS=1
tensorflowjs_converter --input_format=tfjs_layers_model --output_format=keras "$work/tfjs/model.json" "$work/keras.h5"
python -m tf2onnx.convert --keras "$work/keras.h5" --output "$(dirname "$0")/../model/nsfw.onnx" --opset 13
