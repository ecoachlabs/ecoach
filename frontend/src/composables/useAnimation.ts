import { ref } from 'vue'

export function useAnimation() {
  const isAnimating = ref(false)

  function fadeIn(el: HTMLElement, duration: number = 400) {
    el.style.opacity = '0'
    el.style.transform = 'translateY(8px)'
    el.style.transition = `opacity ${duration}ms var(--ease-out), transform ${duration}ms var(--ease-out)`
    requestAnimationFrame(() => {
      el.style.opacity = '1'
      el.style.transform = 'translateY(0)'
    })
  }

  function scaleIn(el: HTMLElement, duration: number = 300) {
    el.style.opacity = '0'
    el.style.transform = 'scale(0.95)'
    el.style.transition = `opacity ${duration}ms var(--ease-spring), transform ${duration}ms var(--ease-spring)`
    requestAnimationFrame(() => {
      el.style.opacity = '1'
      el.style.transform = 'scale(1)'
    })
  }

  function celebrate(el: HTMLElement) {
    el.style.transition = 'transform 300ms cubic-bezier(0.34,1.56,0.64,1)'
    el.style.transform = 'scale(1.05)'
    setTimeout(() => { el.style.transform = 'scale(1)' }, 300)
  }

  function shake(el: HTMLElement) {
    el.style.transition = 'transform 80ms'
    el.style.transform = 'translateX(4px)'
    setTimeout(() => { el.style.transform = 'translateX(-4px)' }, 80)
    setTimeout(() => { el.style.transform = 'translateX(2px)' }, 160)
    setTimeout(() => { el.style.transform = 'translateX(0)' }, 240)
  }

  function pulseGlow(el: HTMLElement, color: string = 'var(--accent)', duration: number = 1000) {
    el.style.transition = `box-shadow ${duration}ms`
    el.style.boxShadow = `0 0 20px ${color}40`
    setTimeout(() => { el.style.boxShadow = 'none' }, duration)
  }

  return { isAnimating, fadeIn, scaleIn, celebrate, shake, pulseGlow }
}
