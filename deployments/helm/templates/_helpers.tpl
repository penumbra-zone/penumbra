{{/*
Expand the name of the chart.
*/}}
{{- define "penumbra.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.

We interpolate the name of the network, e.g. "testnet" or "testnet-preview",
in the fullname so that we can run multiple deployments side-by-side.
*/}}
{{- define "penumbra.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name .Values.network | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "penumbra.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels for resources. See full details at
https://helm.sh/docs/chart_best_practices/labels/
*/}}
{{- define "penumbra.labels" -}}
app.kubernetes.io/name: {{ include "penumbra.name" . }}
helm.sh/chart: {{ include "penumbra.chart" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
{{- end }}

{{/*
Tendermint name.
*/}}
{{- define "tendermint.name" -}}
{{ template "penumbra.fullname" . }}-tm
{{- end -}}
