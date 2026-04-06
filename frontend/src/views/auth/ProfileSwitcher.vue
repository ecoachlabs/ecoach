<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppModal from '@/components/ui/AppModal.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import * as identityIpc from '@/ipc/identity'
import { PIN_LENGTH, isValidPin } from '@/utils/validation'

const auth = useAuthStore()
const router = useRouter()
const loaded = ref(false)
const showCreateModal = ref(false)
const createLoading = ref(false)
const createError = ref('')
const newName = ref('')
const newType = ref('student')
const newPin = ref('')

onMounted(async () => {
  await auth.loadAccounts()
  loaded.value = true
})

function selectProfile(id: number) {
  router.push({ name: 'pin', params: { accountId: String(id) } })
}

function openCreateModal() {
  createError.value = ''
  showCreateModal.value = true
}

function closeCreateModal() {
  showCreateModal.value = false
  createLoading.value = false
  createError.value = ''
  newName.value = ''
  newType.value = 'student'
  newPin.value = ''
}

function getErrorMessage(error: unknown) {
  return typeof error === 'string'
    ? error
    : (error as { message?: string } | null)?.message ?? 'Could not create account.'
}

async function createAccount() {
  if (createLoading.value) return
  const displayName = newName.value.trim()
  const submittedPin = newPin.value.trim()
  if (!displayName) { createError.value = 'Enter a display name.'; return }
  if (!submittedPin) { createError.value = 'Enter a PIN.'; return }
  if (!isValidPin(submittedPin)) {
    createError.value = `PIN must be exactly ${PIN_LENGTH} digits.`
    return
  }
  createLoading.value = true
  createError.value = ''
  try {
    const account = await identityIpc.createAccount({
      account_type: newType.value,
      display_name: displayName,
      pin: submittedPin,
      entitlement_tier: 'standard',
    })
    await auth.loadAccounts()
    closeCreateModal()
    router.push({ name: 'pin', params: { accountId: String(account.id) } })
  } catch (error) {
    createError.value = getErrorMessage(error)
  } finally {
    createLoading.value = false
  }
}

const greeting = computed(() => {
  const h = new Date().getHours()
  if (h < 12) return 'Good morning.'
  if (h < 17) return 'Good afternoon.'
  return 'Good evening.'
})

// Per-account avatar colors — deterministic from position
const avatarPalette = [
  'linear-gradient(135deg, #7C3AED, #A855F7)',  // violet
  'linear-gradient(135deg, #92400E, #B45309)',  // terracotta brown
  'linear-gradient(135deg, #166534, #22C55E)',  // forest green
  'linear-gradient(135deg, #9A3412, #C2410C)',  // dark terracotta
  'linear-gradient(135deg, #1D4ED8, #3B82F6)',  // blue
  'linear-gradient(135deg, #BE185D, #EC4899)',  // pink
  'linear-gradient(135deg, #0F766E, #0D9488)',  // teal
]

function avatarGradient(idx: number): string {
  return avatarPalette[idx % avatarPalette.length]
}

// Floating math symbols
const mathSymbols = [
  { char: 'π', x: 7, y: 11, size: 52, rot: -15 },
  { char: 'Σ', x: 87, y: 8, size: 56, rot: 10 },
  { char: 'θ', x: 51, y: 5, size: 44, rot: 5 },
  { char: '∫', x: 4, y: 62, size: 58, rot: -8 },
  { char: '∞', x: 82, y: 54, size: 42, rot: 0 },
  { char: '√', x: 92, y: 33, size: 50, rot: 12 },
  { char: 'φ', x: 37, y: 63, size: 34, rot: -20 },
  { char: 'λ', x: 14, y: 83, size: 44, rot: 8 },
  { char: 'Ω', x: 73, y: 78, size: 38, rot: -5 },
  { char: '∮', x: 59, y: 87, size: 36, rot: 15 },
  { char: '∝', x: 24, y: 34, size: 30, rot: -12 },
  { char: 'Δ', x: 46, y: 81, size: 32, rot: 18 },
  { char: '⊕', x: 76, y: 17, size: 28, rot: 0 },
  { char: '∇', x: 11, y: 47, size: 34, rot: 6 },
]
</script>

<template>
  <div class="root-bg min-h-screen flex flex-col items-center justify-center relative overflow-hidden">

    <!-- Animated gradient overlay for wave effect -->
    <div class="wave-overlay absolute inset-0 pointer-events-none" />

    <!-- Floating math symbols -->
    <div class="symbols-layer" aria-hidden="true">
      <span
        v-for="(sym, i) in mathSymbols"
        :key="i"
        class="math-sym"
        :style="{
          left: sym.x + '%',
          top: sym.y + '%',
          fontSize: sym.size + 'px',
          '--rot': sym.rot + 'deg',
          animationDelay: (i * 0.35) + 's',
          animationDuration: (6 + i * 0.3) + 's',
        }"
      >{{ sym.char }}</span>
    </div>

    <!-- App wordmark — dark text -->
    <div class="absolute top-5 left-6 flex items-center gap-2.5">
      <div class="w-9 h-9 rounded-xl flex items-center justify-center"
        style="background: rgba(255,255,255,0.22); backdrop-filter: blur(8px); border: 1px solid rgba(255,255,255,0.25); box-shadow: 0 2px 8px rgba(0,0,0,0.08);">
        <span class="font-display font-black text-base" style="color: rgba(60,20,0,0.75);">e</span>
      </div>
      <div>
        <p class="font-display font-bold text-sm leading-none" style="color: rgba(30,10,0,0.80);">Adeo</p>
        <p class="text-[9px] uppercase tracking-widest mt-0.5" style="color: rgba(30,10,0,0.45);">Exam Prep</p>
      </div>
    </div>

    <!-- Center content -->
    <div class="relative z-10 flex flex-col items-center w-full px-6">

      <!-- Greeting — dark text matching screenshot -->
      <h1 class="font-display text-5xl font-bold mb-2 greeting-text"
        style="color: rgba(20,7,0,0.82); letter-spacing: -0.02em;">
        {{ greeting }}
      </h1>
      <p class="text-base font-medium mb-10" style="color: rgba(20,7,0,0.50);">
        Who's studying today?
      </p>

      <!-- Profile container -->
      <div class="profile-container">
        <!-- Skeleton loading -->
        <template v-if="!loaded">
          <div v-for="i in 3" :key="i" class="flex flex-col items-center gap-3 px-4 py-4">
            <div class="w-20 h-20 rounded-full animate-pulse" style="background: rgba(255,255,255,0.25);" />
            <div class="w-16 h-2.5 rounded-full animate-pulse" style="background: rgba(255,255,255,0.2);" />
          </div>
        </template>

        <!-- Profile buttons -->
        <template v-else-if="auth.accounts.length > 0">
          <button
            v-for="(account, i) in auth.accounts"
            :key="account.id"
            class="profile-btn"
            @click="selectProfile(account.id)"
          >
            <!-- Avatar with hover ring -->
            <div class="relative">
              <!-- Ring wrapper — ring animates in on hover -->
              <div class="avatar-ring-wrap" :class="account.account_type">

                <!-- Parent: near-black with permanent golden aura -->
                <div
                  v-if="account.account_type === 'parent'"
                  class="avatar w-20 h-20 rounded-full flex items-center justify-center text-3xl font-bold text-white"
                  style="background: linear-gradient(145deg, #1a1a1a, #2d2d2d);"
                >
                  {{ account.display_name.charAt(0).toUpperCase() }}
                </div>
                <!-- Admin -->
                <div
                  v-else-if="account.account_type === 'admin'"
                  class="avatar w-20 h-20 rounded-full flex items-center justify-center text-3xl font-bold text-white"
                  style="background: linear-gradient(145deg, #1e1b4b, #3730a3);"
                >
                  {{ account.display_name.charAt(0).toUpperCase() }}
                </div>
                <!-- Student -->
                <div
                  v-else
                  class="avatar w-20 h-20 rounded-full flex items-center justify-center text-3xl font-bold text-white"
                  :style="{ background: avatarGradient(i) }"
                >
                  {{ account.display_name.charAt(0).toUpperCase() }}
                </div>
              </div>

              <!-- Parent badge -->
              <div
                v-if="account.account_type === 'parent'"
                class="absolute -bottom-1 -right-1 w-6 h-6 rounded-full flex items-center justify-center text-xs"
                style="background: rgba(30,10,0,0.55); border: 1.5px solid rgba(255,255,255,0.35); z-index: 2; color: white;"
              >P</div>
            </div>

            <!-- Name & meta — dark text -->
            <div class="text-center mt-1">
              <p class="text-sm font-semibold leading-tight" style="color: rgba(20,7,0,0.80);">
                {{ account.display_name }}
              </p>
              <p class="text-[11px] capitalize mt-0.5 font-medium" style="color: rgba(20,7,0,0.45);">
                {{ account.account_type }}
              </p>
              <p v-if="account.last_active_label" class="text-[10px] mt-0.5" style="color: rgba(20,7,0,0.35);">
                {{ account.last_active_label }}
              </p>
            </div>
          </button>

          <!-- Add profile -->
          <button class="profile-btn add-profile-btn" @click="openCreateModal">
            <div
              class="w-20 h-20 rounded-full flex items-center justify-center text-2xl"
              style="border: 1.5px dashed rgba(30,10,0,0.22); color: rgba(30,10,0,0.30);"
            >+</div>
            <div class="text-center mt-1">
              <p class="text-[12px] font-medium mt-0.5" style="color: rgba(20,7,0,0.38);">Add profile</p>
            </div>
          </button>
        </template>

        <!-- Empty state -->
        <div v-else class="flex flex-col items-center py-8 gap-4 w-full">
          <div class="w-20 h-20 rounded-full flex items-center justify-center text-3xl"
            style="background: rgba(255,255,255,0.18); color: rgba(20,7,0,0.4);">+</div>
          <div class="text-center">
            <h2 class="font-display text-xl font-semibold mb-1" style="color: rgba(20,7,0,0.75);">Welcome to eCoach</h2>
            <p class="text-sm mb-4" style="color: rgba(20,7,0,0.45);">Create your first account to get started.</p>
            <button
              class="px-6 py-2.5 rounded-full font-semibold text-sm shadow-md"
              style="background: rgba(30,10,0,0.75); color: white;"
              @click="openCreateModal"
            >Get Started</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom tagline — dark -->
    <p class="absolute bottom-6 text-[10px] uppercase tracking-[0.22em] font-semibold"
      style="color: rgba(20,7,0,0.32);">
      Where Futures Are Built
    </p>

    <!-- Create account modal -->
    <AppModal :open="showCreateModal" title="Create Account" @close="closeCreateModal">
      <div class="space-y-3">
        <AppInput v-model="newName" label="Display Name" placeholder="e.g. Ama Mensah" />
        <AppSelect
          v-model="newType"
          label="Account Type"
          :options="[
            { value: 'student', label: 'Student' },
            { value: 'parent', label: 'Parent' },
            { value: 'admin', label: 'Admin' },
          ]"
        />
        <AppInput v-model="newPin" label="PIN" type="password" placeholder="4 digits" />
        <p v-if="createError" class="text-sm" :style="{ color: 'var(--warm)' }">{{ createError }}</p>
      </div>
      <template #footer>
        <AppButton variant="ghost" @click="closeCreateModal">Cancel</AppButton>
        <AppButton
          variant="primary"
          :loading="createLoading"
          :disabled="!newName.trim() || !newPin.trim()"
          @click="createAccount"
        >Create</AppButton>
      </template>
    </AppModal>
  </div>
</template>

<style scoped>
/* ── Animated gradient background — slow waving flag ── */
.root-bg {
  background:
    linear-gradient(
      145deg,
      #C96212 0%,
      #E87A1E 20%,
      #F5A332 45%,
      #F0B455 65%,
      #E88020 80%,
      #C96212 100%
    );
  background-size: 300% 300%;
  animation: gradient-wave 14s ease-in-out infinite;
}

@keyframes gradient-wave {
  0%   { background-position: 0% 30%; }
  25%  { background-position: 60% 70%; }
  50%  { background-position: 100% 40%; }
  75%  { background-position: 40% 80%; }
  100% { background-position: 0% 30%; }
}

/* ── Secondary wave overlay for extra depth ── */
.wave-overlay {
  background: radial-gradient(
    ellipse at 70% 30%,
    rgba(255, 200, 80, 0.18) 0%,
    rgba(255, 130, 10, 0.08) 45%,
    transparent 70%
  );
  animation: overlay-drift 18s ease-in-out infinite alternate;
}

@keyframes overlay-drift {
  from { transform: translate(0, 0) scale(1); opacity: 0.6; }
  to   { transform: translate(-5%, 5%) scale(1.1); opacity: 1; }
}

/* ── Floating math symbols ── */
.symbols-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  overflow: hidden;
}

.math-sym {
  position: absolute;
  color: rgba(80, 30, 0, 1);   /* solid dark — opacity on the element controls visibility */
  font-family: 'Georgia', 'Times New Roman', serif;
  font-weight: 400;
  line-height: 1;
  user-select: none;
  opacity: 0.11;               /* resting state — clearly faint but present */
  animation: sym-drift ease-in-out infinite alternate;
}

@keyframes sym-drift {
  from { transform: rotate(var(--rot, 0deg)) translateY(0px);   opacity: 0.10; }
  to   { transform: rotate(var(--rot, 0deg)) translateY(-14px); opacity: 0.18; }
}

/* ── Greeting entrance ── */
.greeting-text {
  animation: greet-in 0.75s cubic-bezier(0.16, 1, 0.3, 1) both;
}
@keyframes greet-in {
  from { opacity: 0; transform: translateY(22px); }
  to   { opacity: 1; transform: translateY(0); }
}

/* ── Profile card container — true frosted glass ── */
.profile-container {
  display: flex;
  align-items: flex-start;
  gap: 0;
  padding: 20px 20px;
  border-radius: 26px;

  /*
   * The glass needs to be MEANINGFULLY lighter than the orange behind it.
   * 0.42 white base is the floor — anything less disappears into the bg.
   * Top-left corner is slightly brighter (light source).
   */
  background:
    linear-gradient(
      145deg,
      rgba(255, 255, 255, 0.28) 0%,
      rgba(255, 230, 170, 0.14) 55%,
      rgba(255, 255, 255, 0.22) 100%
    );

  backdrop-filter: blur(32px) saturate(200%) brightness(1.06);
  -webkit-backdrop-filter: blur(32px) saturate(200%) brightness(1.06);

  /* Bright crisp glass edge — especially strong on top and left */
  border: 1px solid rgba(255, 255, 255, 0.70);

  box-shadow:
    /* Neutral lift shadow — no warm tint */
    0 16px 48px rgba(0, 0, 0, 0.14),
    0 4px 16px rgba(0, 0, 0, 0.08),
    /* Strong inner top-edge highlight — glass surface catching overhead light */
    inset 0 2px 0 rgba(255, 255, 255, 0.80),
    /* Subtle inner side highlights */
    inset 1px 0 0 rgba(255, 255, 255, 0.35),
    /* Glass thickness — bottom inner shadow */
    inset 0 -1px 0 rgba(0, 0, 0, 0.06);

  flex-wrap: wrap;
  justify-content: center;
  max-width: 740px;
  width: 100%;
  animation: greet-in 0.75s 0.12s cubic-bezier(0.16, 1, 0.3, 1) both;
  position: relative;
  overflow: hidden;
}

/* Diagonal light sheen — brighter now that we have contrast to work with */
.profile-container::before {
  content: '';
  position: absolute;
  top: 0;
  left: -20%;
  width: 55%;
  height: 100%;
  background: linear-gradient(
    108deg,
    transparent 25%,
    rgba(255, 255, 255, 0.14) 50%,
    transparent 75%
  );
  pointer-events: none;
  border-radius: inherit;
}

/* ── Individual profile button ── */
.profile-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
  padding: 14px 16px 12px;
  border-radius: 18px;
  border: none;
  background: transparent;
  cursor: pointer;
  min-width: 100px;
  /* No transform — ring is the only hover signal */
  transition: background 180ms ease;
}
.profile-btn:hover {
  background: rgba(255, 255, 255, 0.08);
}
.profile-btn:active {
  background: rgba(255, 255, 255, 0.05);
}

.add-profile-btn:hover {
  background: rgba(0, 0, 0, 0.03);
}

/* ── Avatar ring wrap ── */
.avatar-ring-wrap {
  position: relative;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

/* Ring as pseudo-element — hidden by default */
.avatar-ring-wrap::after {
  content: '';
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2.5px solid transparent;
  box-shadow: 0 0 0 0 transparent;
  transition:
    border-color 200ms ease,
    box-shadow 200ms ease,
    inset 200ms ease;
  pointer-events: none;
}

/* Student ring: clean white on hover */
.profile-btn:hover .avatar-ring-wrap::after {
  border-color: rgba(255, 255, 255, 0.75);
  box-shadow: 0 0 12px rgba(255, 255, 255, 0.30);
}

/* Parent ring: same white ring as students on hover — no glow */
.profile-btn:hover .avatar-ring-wrap.parent::after {
  border-color: rgba(255, 255, 255, 0.75);
  box-shadow: 0 0 12px rgba(255, 255, 255, 0.30);
}

/* Admin ring: violet on hover */
.profile-btn:hover .avatar-ring-wrap.admin::after {
  border-color: rgba(167, 139, 250, 0.80);
  box-shadow: 0 0 14px rgba(167, 139, 250, 0.40);
}

/* Avatar — no drop shadow, inset only to keep edge definition */
.avatar {
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.18);
}
</style>
