import { ipc } from '.'

export function getLibraryHome(studentId: number): Promise<any> {
  return ipc('get_library_home', { studentId })
}

export function saveLibraryItem(input: any): Promise<any> {
  return ipc('save_library_item', { input })
}

export function searchGlossary(query: string): Promise<any[]> {
  return ipc('search_glossary', { query })
}
