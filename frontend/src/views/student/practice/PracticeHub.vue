<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const router = useRouter()

const subjects = ref([
  { id: 1, code: 'MATH', name: 'Mathematics', topicCount: 18, questionCount: 240, icon: '∑' },
  { id: 2, code: 'ENG', name: 'English Language', topicCount: 15, questionCount: 180, icon: 'Aa' },
  { id: 3, code: 'SCI', name: 'Integrated Science', topicCount: 16, questionCount: 200, icon: '⚛' },
])

const recentSessions = ref([
  { id: 1, subject: 'Mathematics', topic: 'Fractions', accuracy: 72, questions: 10, date: '2 hours ago' },
  { id: 2, subject: 'English', topic: 'Comprehension', accuracy: 85, questions: 8, date: 'Yesterday' },
])
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="mb-8">
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Practice</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Choose a subject and start practicing. Every question teaches you something.</p>
    </div>

    <!-- Subject Cards -->
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
      <AppCard v-for="subject in subjects" :key="subject.id" hover padding="lg"
        @click="router.push('/student/practice/custom-test')">
        <div class="flex items-center gap-3 mb-4">
          <div class="w-11 h-11 rounded-xl flex items-center justify-center text-lg font-display font-bold"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
            {{ subject.icon }}
          </div>
          <div>
            <h3 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ subject.name }}</h3>
            <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ subject.topicCount }} topics · {{ subject.questionCount }} questions</p>
          </div>
        </div>
        <AppButton variant="secondary" size="sm" class="w-full">Start Practice →</AppButton>
      </AppCard>
    </div>

    <!-- Quick Actions -->
    <div class="flex flex-wrap gap-2 mb-8">
      <AppButton variant="primary" size="sm" @click="router.push('/student/practice/custom-test')">✎ Custom Test</AppButton>
      <AppButton variant="secondary" size="sm">◎ Weakness Focus</AppButton>
      <AppButton variant="secondary" size="sm">⏱ Timed Drill</AppButton>
      <AppButton variant="secondary" size="sm">↻ Review Mistakes</AppButton>
    </div>

    <!-- Recent Sessions -->
    <div>
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Recent Sessions</h3>
      <div class="space-y-2">
        <AppCard v-for="session in recentSessions" :key="session.id" padding="sm" hover>
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold"
              :class="session.accuracy >= 80 ? 'bg-emerald-50 text-emerald-600' : session.accuracy >= 60 ? 'bg-amber-50 text-amber-600' : 'bg-red-50 text-red-600'">
              {{ session.accuracy }}%
            </div>
            <div class="flex-1">
              <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ session.topic }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ session.subject }} · {{ session.questions }} questions</p>
            </div>
            <span class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ session.date }}</span>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
