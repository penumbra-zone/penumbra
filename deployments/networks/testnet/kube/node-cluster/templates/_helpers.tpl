{{/*
Penumbra Daemon name.
*/}}
{{- define "penumbra.name" -}}
{{ .Values.network }}-pd-{{ .Values.name }}
{{- end -}}


{{/*
Tendermint name.
*/}}
{{- define "tendermint.name" -}}
{{ .Values.network }}-tm-{{ .Values.name }}
{{- end -}}
