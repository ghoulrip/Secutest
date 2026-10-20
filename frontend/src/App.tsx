import { useState, useEffect } from 'react'
import ScanPage from './pages/ScanPage'
import ReportPage from './pages/ReportPage'
import './App.css'

type View = 'scan' | 'report'

export default function App() {
  const [view, setView] = useState<View>('scan')
  const [selectedScanId, setSelectedScanId] = useState<string | null>(null)

  // Handle deep-link to a specific scan report via URL hash.
  useEffect(() => {
    const hash = window.location.hash.replace('#', '')
    if (hash.startsWith('scan=')) {
      setSelectedScanId(hash.slice(5))
      setView('report')
    }
  }, [])

  const handleViewReport = (scanId: string) => {
    setSelectedScanId(scanId)
    setView('report')
    window.location.hash = `scan=${scanId}`
  }

  const handleBack = () => {
    setView('scan')
    setSelectedScanId(null)
    window.location.hash = ''
  }

  return (
    <div className="app-shell">
      <nav className="nav">
        <button className="nav-brand" onClick={() => { setView('scan'); window.location.hash = '' }}>
          🔍 Secutest
        </button>
        <div className="nav-links">
          <a
            href="#"
            className={view === 'scan' ? 'nav-link active' : 'nav-link'}
            onClick={(e) => { e.preventDefault(); handleBack() }}
          >
            Scan
          </a>
          {selectedScanId && (
            <a
              href="#"
              className={view === 'report' ? 'nav-link active' : 'nav-link'}
              onClick={(e) => { e.preventDefault(); handleViewReport(selectedScanId) }}
            >
              Report
            </a>
          )}
        </div>
      </nav>

      <main className="main-content">
        {view === 'scan' ? (
          <ScanPage onScanComplete={handleViewReport} />
        ) : selectedScanId ? (
          <ReportPage scanId={selectedScanId} onBack={handleBack} />
        ) : null}
      </main>

      <footer className="app-footer">
        <p>Secutest · Leak Detection · React + Vite</p>
      </footer>
    </div>
  )
}