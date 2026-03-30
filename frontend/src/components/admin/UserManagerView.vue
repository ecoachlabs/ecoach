<script setup lang="ts">
import { ref, onMounted } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppTable from '@/components/ui/AppTable.vue'
import AppModal from '@/components/ui/AppModal.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import { useAuthStore } from '@/stores/auth'
import * as identityIpc from '@/ipc/identity'

const auth = useAuthStore()
const accounts = ref<any[]>([])
const loading = ref(true)
const showCreateModal = ref(false)
const newName = ref('')
const newType = ref('student')
const newPin = ref('')

onMounted(async () => {
  await auth.loadAccounts()
  accounts.value = auth.accounts
  loading.value = false
})

async function createAccount() {
  if (!newName.value.trim() || !newPin.value) return
  try {
    await identityIpc.createAccount({
      account_type: newType.value,
      display_name: newName.value.trim(),
      pin: newPin.value,
      entitlement_tier: 'standard',
    })
    await auth.loadAccounts()
    accounts.value = auth.accounts
    showCreateModal.value = false
    newName.value = ''; newPin.value = ''
  } catch {}
}

const columns = [
  { key: 'display_name', label: 'Name' },
  { key: 'account_type', label: 'Role', width: '100px' },
  { key: 'status', label: 'Status', width: '100px' },
  { key: 'last_active_label', label: 'Last Active', width: '120px' },
]
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-base font-bold" :style="{ color: 'var(--text)' }">User Management</h2>
      <AppButton variant="primary" size="sm" @click="showCreateModal = true">+ Create Account</AppButton>
    </div>

    <AppTable :columns="columns" :rows="accounts" compact>
      <template #account_type="{ value }">
        <AppBadge :color="value === 'admin' ? 'accent' : value === 'parent' ? 'gold' : 'muted'" size="xs">{{ value }}</AppBadge>
      </template>
      <template #status="{ value }">
        <AppBadge :color="value === 'active' ? 'success' : 'muted'" size="xs">{{ value }}</AppBadge>
      </template>
    </AppTable>

    <AppModal :open="showCreateModal" title="Create Account" @close="showCreateModal = false">
      <div class="space-y-3">
        <AppInput v-model="newName" label="Display Name" placeholder="e.g. Ama Mensah" />
        <AppSelect v-model="newType" label="Account Type" :options="[{value:'student',label:'Student'},{value:'parent',label:'Parent'},{value:'admin',label:'Admin'}]" />
        <AppInput v-model="newPin" label="PIN" type="password" placeholder="4-6 digits" />
      </div>
      <template #footer>
        <AppButton variant="ghost" @click="showCreateModal = false">Cancel</AppButton>
        <AppButton variant="primary" @click="createAccount">Create</AppButton>
      </template>
    </AppModal>
  </div>
</template>
