<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'

defineProps<{
  messages: { from: 'parent' | 'coach'; text: string; time: string }[]
}>()

defineEmits<{ send: [message: string] }>()

const input = ref('')
function send() {
  if (input.value.trim()) { emit('send', input.value.trim()); input.value = '' }
}
const emit = defineEmits<{ send: [message: string] }>()
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex-1 overflow-y-auto p-4 space-y-3">
      <div v-for="(msg, i) in messages" :key="i" :class="msg.from === 'parent' ? 'flex justify-end' : 'flex justify-start'">
          <div class="max-w-[80%] px-4 py-2.5 rounded-[var(--radius-lg)]"
          :class="msg.from === 'parent' ? 'rounded-br-sm' : 'rounded-bl-sm'"
          :style="{
            backgroundColor: msg.from === 'parent' ? 'var(--accent)' : 'var(--card-bg)',
            color: msg.from === 'parent' ? 'white' : 'var(--text)',
            boxShadow: msg.from === 'coach' ? 'var(--shadow-xs)' : 'none',
          }">
          <p class="text-sm leading-relaxed">{{ msg.text }}</p>
          <p class="text-[9px] mt-1 opacity-60">{{ msg.time }}</p>
        </div>
      </div>
      <p v-if="!messages.length" class="text-sm text-center py-12" :style="{color:'var(--text-3)'}">
        Ask a question about your child's academic progress.
      </p>
    </div>
    <div class="shrink-0 p-3 border-t flex gap-2" :style="{borderColor:'var(--card-border)'}">
      <input v-model="input" placeholder="Ask about your child's progress..."
        class="flex-1 px-4 py-2.5 rounded-[var(--radius-lg)] border text-sm"
        :style="{borderColor:'var(--card-border)',backgroundColor:'var(--card-bg)',color:'var(--text)'}"
        @keydown.enter="send" />
      <AppButton variant="primary" :disabled="!input.trim()" @click="send">Send</AppButton>
    </div>
  </div>
</template>
