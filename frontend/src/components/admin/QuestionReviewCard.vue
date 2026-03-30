<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  questionId: number
  stem: string
  classification: string
  confidence: number
  status: 'pending' | 'approved' | 'rejected'
}>()

defineEmits<{ approve: []; reject: []; edit: [] }>()
</script>

<template>
  <AppCard padding="md">
    <div class="flex items-start gap-3">
      <div class="w-8 h-8 rounded-lg flex items-center justify-center text-[10px] font-bold shrink-0"
        :class="status === 'approved' ? 'bg-emerald-100 text-emerald-700' : status === 'rejected' ? 'bg-red-100 text-red-600' : ''"
        :style="status === 'pending' ? {backgroundColor:'var(--primary-light)',color:'var(--text-3)'} : {}">
        #{{ questionId }}
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm mb-1 line-clamp-2" :style="{color:'var(--text)'}">{{ stem }}</p>
        <div class="flex items-center gap-2">
          <AppBadge color="accent" size="xs">{{ classification }}</AppBadge>
          <span class="text-[10px] tabular-nums" :style="{color: confidence >= 80 ? 'var(--success)' : confidence >= 50 ? 'var(--gold)' : 'var(--danger)'}">
            {{ confidence }}% confidence
          </span>
        </div>
      </div>
      <div v-if="status === 'pending'" class="flex gap-1 shrink-0">
        <AppButton variant="ghost" size="sm" @click="$emit('approve')">✓</AppButton>
        <AppButton variant="ghost" size="sm" @click="$emit('reject')">✕</AppButton>
        <AppButton variant="ghost" size="sm" @click="$emit('edit')">✎</AppButton>
      </div>
    </div>
  </AppCard>
</template>
