{{/*
Standard Helm helpers for cc-container generated charts.
*/}}

{{/*
Chart name, truncated to 63 chars (K8s label limit).
*/}}
{{- define "chart.name" -}}
{{- .Chart.Name | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Fully qualified app name: release-name + chart-name.
If release name contains chart name, it is used as-is.
Truncated to 63 chars.
*/}}
{{- define "chart.fullname" -}}
{{- if contains .Chart.Name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{/*
Standard Kubernetes labels.
*/}}
{{- define "chart.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{ include "chart.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels (used in both labels and matchLabels).
*/}}
{{- define "chart.selectorLabels" -}}
app.kubernetes.io/name: {{ include "chart.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Build a full image reference from an image object with optional registry.
*/}}
{{- define "chart.imageRef" -}}
{{- if .registry }}{{ .registry }}/{{ end }}{{ .repository }}:{{ .tag }}
{{- end }}
