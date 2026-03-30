import { ref } from 'vue'

export function usePdf() {
  const generating = ref(false)
  const error = ref('')

  async function exportToPdf(elementId: string, filename: string = 'report.pdf') {
    generating.value = true
    error.value = ''

    try {
      // Dynamic import jsPDF when needed
      const [{ default: jsPDF }, { default: html2canvas }] = await Promise.all([
        import('jspdf').catch(() => ({ default: null })),
        import('html2canvas').catch(() => ({ default: null })),
      ])

      if (!jsPDF || !html2canvas) {
        // Fallback to print
        window.print()
        return
      }

      const element = document.getElementById(elementId)
      if (!element) {
        error.value = 'Element not found'
        return
      }

      const canvas = await html2canvas(element, {
        scale: 2,
        useCORS: true,
        logging: false,
      })

      const imgData = canvas.toDataURL('image/png')
      const pdf = new jsPDF('p', 'mm', 'a4')
      const pdfWidth = pdf.internal.pageSize.getWidth()
      const pdfHeight = (canvas.height * pdfWidth) / canvas.width

      pdf.addImage(imgData, 'PNG', 0, 0, pdfWidth, pdfHeight)
      pdf.save(filename)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to generate PDF'
      // Fallback to print
      window.print()
    } finally {
      generating.value = false
    }
  }

  return { generating, error, exportToPdf }
}
