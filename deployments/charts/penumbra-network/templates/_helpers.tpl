{{/*
Expand the name of the chart.
*/}}
{{- define "penumbra-network.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "penumbra-network.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- if .Values.network.chain_id }}
{{- printf "%s-%s" .Release.Name .Values.network.chain_id | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s" .Release.Name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "penumbra-network.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "penumbra-network.part_of" }}
{{- if .Values.part_of }}
{{- printf "%s" .Values.part_of }}
{{- else }}
{{- printf "%s" .Release.Name }}
{{- end }}
{{- end }}

{{- define "penumbra-network.labels" -}}
helm.sh/chart: {{ include "penumbra-network.chart" . }}
{{ include "penumbra-network.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: {{ include "penumbra-network.part_of" . }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "penumbra-network.selectorLabels" -}}
app.kubernetes.io/name: {{ include "penumbra-network.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "penumbra-network.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "penumbra-network.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
