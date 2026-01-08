{{/*
Expand the name of the chart.
*/}}
{{- define "role-panel-bot.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "role-panel-bot.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "role-panel-bot.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "role-panel-bot.labels" -}}
helm.sh/chart: {{ include "role-panel-bot.chart" . }}
{{ include "role-panel-bot.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "role-panel-bot.selectorLabels" -}}
app.kubernetes.io/name: {{ include "role-panel-bot.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "role-panel-bot.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "role-panel-bot.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Database URL
*/}}
{{- define "role-panel-bot.databaseUrl" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "postgres://%s:$(POSTGRES_PASSWORD)@%s-postgresql:5432/%s" .Values.postgresql.auth.username (include "role-panel-bot.fullname" .) .Values.postgresql.auth.database }}
{{- else }}
{{- printf "postgres://%s:$(POSTGRES_PASSWORD)@%s:%d/%s" .Values.externalDatabase.username .Values.externalDatabase.host (int .Values.externalDatabase.port) .Values.externalDatabase.database }}
{{- end }}
{{- end }}

{{/*
Discord secret name
*/}}
{{- define "role-panel-bot.discordSecretName" -}}
{{- if .Values.discord.existingSecret }}
{{- .Values.discord.existingSecret }}
{{- else }}
{{- printf "%s-discord" (include "role-panel-bot.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Database secret name
*/}}
{{- define "role-panel-bot.databaseSecretName" -}}
{{- if .Values.postgresql.enabled }}
{{- if .Values.postgresql.auth.existingSecret }}
{{- .Values.postgresql.auth.existingSecret }}
{{- else }}
{{- printf "%s-postgresql" (include "role-panel-bot.fullname" .) }}
{{- end }}
{{- else }}
{{- if .Values.externalDatabase.existingSecret }}
{{- .Values.externalDatabase.existingSecret }}
{{- else }}
{{- printf "%s-database" (include "role-panel-bot.fullname" .) }}
{{- end }}
{{- end }}
{{- end }}
