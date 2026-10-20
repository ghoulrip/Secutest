export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'info'

export type ScanStatus = 'queued' | 'running' | 'completed' | 'failed'

export interface Solution {
  summary: string
  steps: string[]
  references: { title: string; url: string }[]
}

export interface Finding {
  id: string
  title: string
  severity: Severity
  category: string
  description: string
  location?: string
  impact?: string
  evidence?: string[]
  solution?: Solution
}

export interface ScanSummary {
  total: number
  critical: number
  high: number
  medium: number
  low: number
  info: number
  riskScore: number
}

export interface Scan {
  scanId: string
  url: string
  status: ScanStatus
  progress: number
  startedAt: string
  completedAt?: string
  error?: string
  summary?: ScanSummary
  findings: Finding[]
}

export interface ScanRequest {
  url: string
  options?: {
    includeSubdomains?: boolean
    followRedirects?: boolean
    maxDepth?: number
  }
}

export const SEVERITY_ORDER: Severity[] = ['critical', 'high', 'medium', 'low', 'info']

export const SEVERITY_LABEL: Record<Severity, string> = {
  critical: 'Critical',
  high: 'High',
  medium: 'Medium',
  low: 'Low',
  info: 'Info',
}

export function severityLabel(severity: Severity): string {
  return SEVERITY_LABEL[severity] ?? severity
}