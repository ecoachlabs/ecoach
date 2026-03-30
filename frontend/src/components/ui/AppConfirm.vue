<script setup lang="ts">
import AppModal from './AppModal.vue'
import AppButton from './AppButton.vue'

defineProps<{
  open: boolean
  title: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  danger?: boolean
}>()

defineEmits<{ confirm: []; cancel: [] }>()
</script>

<template>
  <AppModal :open="open" :title="title" size="sm" @close="$emit('cancel')">
    <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ message }}</p>
    <template #footer>
      <AppButton variant="ghost" @click="$emit('cancel')">{{ cancelLabel || 'Cancel' }}</AppButton>
      <AppButton :variant="danger ? 'danger' : 'primary'" @click="$emit('confirm')">{{ confirmLabel || 'Confirm' }}</AppButton>
    </template>
  </AppModal>
</template>
