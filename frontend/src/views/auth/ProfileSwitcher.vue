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
const showAdminModal = ref(false)
const adminCode = ref('')
const adminError = ref('')
const adminLoading = ref(false)
const adminAccessCode = import.meta.env.VITE_SUPER_ADMIN_CODE
  ?? localStorage.getItem('ecoach.superAdminCode')
  ?? '2468'

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

const visibleAccounts = computed(() =>
  auth.accounts.filter(account => account.account_type !== 'admin')
)

const adminAccounts = computed(() =>
  auth.accounts.filter(account => account.account_type === 'admin')
)

function openAdminModal() {
  adminCode.value = ''
  adminError.value = ''
  showAdminModal.value = true
}

function closeAdminModal() {
  showAdminModal.value = false
  adminCode.value = ''
  adminError.value = ''
  adminLoading.value = false
}

async function unlockAdmin() {
  if (adminLoading.value) return
  const submittedCode = adminCode.value.trim()
  if (!isValidPin(submittedCode)) {
    adminError.value = `Enter the ${PIN_LENGTH}-digit super admin code.`
    return
  }
  if (submittedCode !== adminAccessCode) {
    adminError.value = 'That code does not unlock admin access.'
    adminCode.value = ''
    return
  }

  adminLoading.value = true
  adminError.value = ''
  try {
    let account = adminAccounts.value[0]
    if (!account) {
      account = await identityIpc.createAccount({
        account_type: 'admin',
        display_name: 'Super Admin',
        pin: submittedCode,
        entitlement_tier: 'elite',
      })
      await auth.loadAccounts()
    }
    await auth.login(account.id, submittedCode)
    closeAdminModal()
    router.push('/admin')
  } catch (error) {
    adminError.value = getErrorMessage(error)
  } finally {
    adminLoading.value = false
  }
}

const greeting = computed(() => {
  const h = new Date().getHours()
  if (h < 12) return 'Good morning.'
  if (h < 17) return 'Good afternoon.'
  return 'Good evening.'
})

// Student avatar palette — vibrant, contrast-rich colors that sing against warm amber.
// No purples, no pinks, no near-black (black is reserved for parent accounts).
const avatarPalette = [
  'linear-gradient(135deg, #1D4ED8, #3B82F6)',  // marine blue
  'linear-gradient(135deg, #059669, #10B981)',  // emerald
  'linear-gradient(135deg, #B91C1C, #DC2626)',  // vermilion
  'linear-gradient(135deg, #0891B2, #06B6D4)',  // cyan
  'linear-gradient(135deg, #166534, #15803D)',  // forest
  'linear-gradient(135deg, #1E3A8A, #2563EB)',  // sapphire
  'linear-gradient(135deg, #BE123C, #E11D48)',  // crimson
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
        <span class="font-display text-lg" style="color: rgba(60,20,0,0.75); line-height: 1;">e</span>
      </div>
      <div>
        <p class="font-display text-base leading-none" style="color: rgba(30,10,0,0.82);">Adeo</p>
        <p class="text-[9px] uppercase tracking-widest mt-0.5" style="color: rgba(30,10,0,0.42); font-weight: 500;">Exam Prep</p>
      </div>
    </div>

    <!-- Center content -->
    <div class="relative z-10 flex flex-col items-center w-full px-6">

      <!-- Greeting — display serif at its natural weight, no forced bold -->
      <h1 class="font-display text-6xl mb-3 greeting-text"
        style="color: rgba(20,7,0,0.85); letter-spacing: -0.015em; line-height: 1;">
        {{ greeting }}
      </h1>
      <p class="text-base mb-10" style="color: rgba(20,7,0,0.48); font-weight: 400;">
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
        <template v-else-if="visibleAccounts.length > 0">
          <button
            v-for="(account, i) in visibleAccounts"
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

              <!-- Parent badge — heraldic gold shield with embossed P -->
              <div
                v-if="account.account_type === 'parent'"
                class="parent-shield"
                aria-label="Parent"
              >
                <span class="parent-shield-letter">P</span>
              </div>
            </div>

            <!-- Name & meta — dark text, light weights -->
            <div class="text-center mt-1">
              <p class="text-sm leading-tight" style="color: rgba(20,7,0,0.82); font-weight: 500;">
                {{ account.display_name }}
              </p>
              <p class="text-[11px] capitalize mt-0.5" style="color: rgba(20,7,0,0.45); font-weight: 400;">
                {{ account.account_type }}
              </p>
              <p v-if="account.last_active_label" class="text-[10px] mt-0.5" style="color: rgba(20,7,0,0.32); font-weight: 400;">
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
              <p class="text-[12px] mt-0.5" style="color: rgba(20,7,0,0.38); font-weight: 400;">Add profile</p>
            </div>
          </button>

          <button class="profile-btn add-profile-btn" @click="openAdminModal">
            <div
              class="w-20 h-20 rounded-full flex items-center justify-center text-xl font-bold"
              style="border: 1.5px solid rgba(30,10,0,0.22); color: rgba(30,10,0,0.42); background: rgba(255,255,255,0.14);"
            >#</div>
            <div class="text-center mt-1">
              <p class="text-[12px] mt-0.5" style="color: rgba(20,7,0,0.42); font-weight: 500;">Under the hood</p>
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
            <button
              class="mt-3 px-5 py-2 rounded-full font-semibold text-xs"
              style="background: rgba(255,255,255,0.18); color: rgba(30,10,0,0.65);"
              @click="openAdminModal"
            >Under the hood</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom tagline — dark -->
    <p class="absolute bottom-6 text-[10px] uppercase tracking-[0.22em]"
      style="color: rgba(20,7,0,0.32); font-weight: 500;">
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

    <AppModal :open="showAdminModal" title="Super Admin Access" @close="closeAdminModal">
      <div class="space-y-3">
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
          Enter the under-the-hood access code to manage content, questions, sources, and system operations.
        </p>
        <AppInput v-model="adminCode" label="Access Code" type="password" placeholder="4 digits" />
        <p v-if="adminError" class="text-sm" :style="{ color: 'var(--warm)' }">{{ adminError }}</p>
      </div>
      <template #footer>
        <AppButton variant="ghost" @click="closeAdminModal">Cancel</AppButton>
        <AppButton
          variant="primary"
          :loading="adminLoading"
          :disabled="!adminCode.trim()"
          @click="unlockAdmin"
        >Unlock</AppButton>
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

/* ── Profile card — cool-tinted glass that stays neutral on warm amber ──
 *
 * The old card went yellow because:
 *   1. Its white tint was too low-alpha, so the orange bg read right through it.
 *   2. saturate(200%) pumped up every warm pixel passing through the blur.
 *
 * This version inverts both: high-alpha cool-tinted white (reads as paper, not
 * as tinted amber) + saturate(75%) to bleach the warmth out of the backdrop.
 */
.profile-container {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  gap: 28px;
  padding: 28px 36px 24px;
  border-radius: 28px;
  flex-wrap: wrap;
  max-width: 720px;
  width: 100%;
  position: relative;

  /* Slight cool tint (252/253/255) counters the warm bleed from behind */
  background:
    linear-gradient(
      180deg,
      rgba(252, 253, 255, 0.18) 0%,
      rgba(248, 250, 253, 0.13) 100%
    );

  backdrop-filter: blur(44px) saturate(65%);
  -webkit-backdrop-filter: blur(44px) saturate(65%);

  /* Double border — inner crisp highlight + outer hairline for refinement */
  border: 1px solid rgba(255, 255, 255, 0.85);

  box-shadow:
    /* Warm-neutral lift shadow — sits on amber without muddying it */
    0 22px 48px rgba(60, 25, 0, 0.16),
    0 6px 14px rgba(60, 25, 0, 0.08),
    /* Crisp inner top highlight — reads as overhead light on glass */
    inset 0 1px 0 rgba(255, 255, 255, 0.95),
    /* Subtle bottom edge to imply thickness */
    inset 0 -1px 0 rgba(60, 25, 0, 0.04);

  animation: greet-in 0.75s 0.12s cubic-bezier(0.16, 1, 0.3, 1) both;
}

/* Whisper-thin diagonal sheen — adds the "caught light" sparkle on glass */
.profile-container::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: linear-gradient(
    115deg,
    transparent 30%,
    rgba(255, 255, 255, 0.22) 50%,
    transparent 70%
  );
  pointer-events: none;
  opacity: 0.5;
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
  position: relative;
  z-index: 1;           /* sits above the container's ::after sheen */
  transition: background 180ms ease;
}
.profile-btn:hover {
  background: rgba(60, 25, 0, 0.045);   /* warm-neutral on white card, readable */
}
.profile-btn:active {
  background: rgba(60, 25, 0, 0.07);
}

.add-profile-btn:hover {
  background: rgba(60, 25, 0, 0.03);
}

/* ── Avatar ring wrap ── */
.avatar-ring-wrap {
  position: relative;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

/* Hover-only medallion rim — a drawn line appears around the avatar on interaction.
 * On a white card surface a cream rim vanishes, so we use a warm-dark hairline. */
.avatar-ring-wrap::before {
  content: '';
  position: absolute;
  inset: -6px;
  border-radius: 50%;
  border: 1px solid transparent;
  pointer-events: none;
  transition: inset 220ms ease, border-color 220ms ease;
}

.profile-btn:hover .avatar-ring-wrap::before {
  inset: -9px;
  border-color: rgba(60, 25, 0, 0.18);
}

/* Legacy hover ring — disabled, the ::before rim is the only hover signal */
.avatar-ring-wrap::after {
  content: '';
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px solid transparent;
  pointer-events: none;
}

/* ── Parent heraldic shield badge ──
 * Classic flat-top, pointed-bottom heraldic shield shape carved via clip-path.
 * Brushed-gold gradient + embossed P (raised letter effect with dual text-shadow). */
.parent-shield {
  position: absolute;
  bottom: -6px;
  right: -6px;
  width: 24px;
  height: 28px;
  z-index: 2;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 4px;

  /* Layered golden gradient — highlight top-left, body mid-gold, deep bronze lower-right.
   * This is what makes it read as metallic instead of flat yellow. */
  background:
    linear-gradient(
      150deg,
      #FFF1BE 0%,
      #F5CE52 22%,
      #D4A017 50%,
      #9A6F12 82%,
      #6E4E0A 100%
    );

  /* Heraldic shield outline: flat top, squared shoulders, pointed chief at bottom */
  clip-path: polygon(
    0% 0%,
    100% 0%,
    100% 60%,
    50% 100%,
    0% 60%
  );

  /* drop-shadow works on the clipped shape (unlike box-shadow which would be clipped away) */
  filter:
    drop-shadow(0 2px 3px rgba(60, 35, 0, 0.45))
    drop-shadow(0 0 0.5px rgba(40, 25, 0, 0.9));
}

/* Embossed P — dark bronze letter with a bright cream top edge (raised-from-metal look) */
.parent-shield-letter {
  font-family: 'Georgia', 'Times New Roman', serif;
  font-weight: 900;
  font-size: 12px;
  line-height: 1;
  color: #4A3209;
  text-shadow:
    0 1px 0 rgba(255, 245, 200, 0.85),     /* bright top edge — catches light */
    0 -0.5px 0 rgba(40, 25, 0, 0.35);      /* faint bottom edge — carves depth */
  letter-spacing: 0;
  user-select: none;
}

/* Avatar — grounded drop shadow on the warm bg + inset highlight for coin-like edge */
.avatar {
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.22),
    inset 0 -1px 0 rgba(0, 0, 0, 0.18),
    0 14px 32px rgba(40, 15, 0, 0.22),
    0 4px 10px rgba(40, 15, 0, 0.12);
  transition: transform 220ms cubic-bezier(0.16, 1, 0.3, 1);
}

.profile-btn:hover .avatar {
  transform: translateY(-2px);
}
</style>
