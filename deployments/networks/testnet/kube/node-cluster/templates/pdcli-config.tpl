apiVersion: v1
kind: ConfigMap
metadata:
  name: "pdfiles-0"
binaryData:
{{ range $path, $bytes := .Files.Glob (printf "pdcli/.penumbra/testnet_data/node0/tendermint/config/**")}}
{{ $name := base $path }}
{{- sha256sum (printf "%s/%s" (index (regexSplit "pdcli/.penumbra/testnet_data/node0/tendermint/config/ " (dir $path) -1) 1 ) $name ) | indent 2 }}{{ print ": "}}{{ $.Files.Get $path | b64enc }}
{{ end }}
