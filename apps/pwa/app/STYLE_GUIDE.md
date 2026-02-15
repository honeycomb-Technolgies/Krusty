# Krusty Frontend Style Guide

## Brand Identity

Krusty is a terminal-first AI coding assistant. The visual language reflects:
- **Dark, metallic aesthetic** - Like a well-used terminal
- **Warm accents** - Orange/gold gradients (the "Krusty" color)
- **Monospace DNA** - ASCII art, code-like elements
- **Subtle motion** - Water-like plasma, shimmer effects

## Color Palette

### Dark Mode (Primary)

```css
--background: 240 10% 3.9%;     /* Near black #0a0a0b */
--foreground: 0 0% 98%;          /* Near white #fafafa */
--card: 240 10% 3.9%;            /* Same as background */
--muted: 240 3.7% 15.9%;         /* Dark gray #262629 */
--muted-foreground: 240 5% 64.9%; /* Medium gray #a1a1a6 */
--border: 240 3.7% 15.9%;        /* Subtle borders */
```

### Accent Colors

```css
--primary: 0 0% 98%;             /* White (buttons, links) */
--user-message: 217 91% 60%;     /* Blue #3b82f6 */
--thinking: 45 93% 47%;          /* Gold/amber #e6a600 */
--destructive: 0 62.8% 30.6%;    /* Deep red */
```

### Brand Gradient (Krusty Orange)

Used for ASCII title and highlights:
```css
background: linear-gradient(
  90deg,
  #8b4513 0%,    /* Saddle brown */
  #cd853f 15%,   /* Peru */
  #ff6b35 35%,   /* Orange */
  #ffcc00 50%,   /* Gold */
  #ff6b35 65%,
  #cd853f 85%,
  #8b4513 100%
);
```

## Typography

### Font Stack

```css
--font-sans: 'Inter', system-ui, sans-serif;
--font-mono: 'JetBrains Mono', 'Fira Code', monospace;
```

### Usage

- **Body text**: Inter, 14-16px
- **Code/terminal**: JetBrains Mono
- **ASCII art**: JetBrains Mono, no ligatures
- **Headings**: Inter, bold

## Components

### Cards

```svelte
<div class="rounded-xl border border-border bg-card p-4">
  <!-- Content -->
</div>
```

Characteristics:
- `rounded-xl` (0.75rem border radius)
- Subtle border `border-border`
- No shadow (flat design)
- Background matches page for glass effect

### Buttons

**Primary (solid)**
```svelte
<button class="rounded-lg bg-primary px-4 py-2 text-primary-foreground hover:bg-primary/90">
  Action
</button>
```

**Secondary (muted)**
```svelte
<button class="rounded-lg bg-muted px-4 py-2 text-muted-foreground hover:text-foreground">
  Secondary
</button>
```

**Destructive**
```svelte
<button class="rounded-lg bg-destructive/10 px-4 py-2 text-destructive hover:bg-destructive/20">
  Delete
</button>
```

### Input Fields

```svelte
<input
  class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm
    placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
/>
```

## Animations

### Standard Keyframes

```css
/* Fade in */
@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Slide in from right */
@keyframes slide-in-right {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

/* Subtle pulse */
@keyframes pulse-subtle {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

/* Message enter */
@keyframes message-fade-in {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

### ASCII Title Animation

The KRUSTY logo uses two combined animations:
1. **Shimmer** - Gradient position slides left-right (4s)
2. **Wave** - Subtle Y translation per line (3s)

Each line is staggered by 0.1-0.15s for wave effect.

### Thinking Pulse

Gold border that pulses when AI is thinking:
```css
@keyframes thinking-pulse {
  0%, 100% {
    opacity: 1;
    border-color: hsl(var(--thinking));
  }
  50% {
    opacity: 0.7;
    border-color: hsl(var(--thinking) / 0.5);
  }
}
```

## Special Effects

### Plasma Background

WebGL shader creating water turbulence effect:
- Dark metallic colors (near black → dark blue → teal)
- 30fps for performance
- Pauses when tab is inactive
- Full viewport, z-index: -100

### Crab Mascot Animation

ASCII art crab with states:
- **Walking**: Legs alternate /\ and ||
- **Looking around**: Eyes shift left/right, blinks
- **Pinching**: Claws open/close

Rendered in JetBrains Mono with the brand gradient.

## Layout Patterns

### Mobile-First Navigation

Bottom nav bar for mobile (like native apps):
```svelte
<nav class="safe-bottom flex h-16 items-center justify-around border-t border-border/50 bg-card/60 backdrop-blur-sm">
  <!-- Nav items -->
</nav>
```

### Content Cards

```svelte
<div class="flex items-center gap-4 rounded-xl border border-border bg-card p-4 hover:bg-muted transition-colors">
  <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
    <Icon class="h-5 w-5 text-muted-foreground" />
  </div>
  <div class="flex-1">
    <div class="font-medium">Title</div>
    <div class="text-sm text-muted-foreground">Description</div>
  </div>
</div>
```

## Icons

Use **Lucide** icons (lucide-svelte):
```svelte
import { MessageSquare, Terminal, Code2, Settings } from 'lucide-svelte';

<MessageSquare class="h-5 w-5" />
```

Standard sizes:
- Navigation: 5x5 (h-5 w-5)
- Inline: 4x4 (h-4 w-4)
- Feature icons: 6x6 to 8x8

## Spacing

Use Tailwind's spacing scale:
- Tight: gap-2, p-2
- Normal: gap-4, p-4
- Relaxed: gap-6, p-6
- Sections: gap-8, py-8

## Do's and Don'ts

### Do
- Use the plasma background on main pages
- Apply brand gradient to ASCII art
- Keep buttons rounded-lg or rounded-xl
- Use monospace for anything code-related
- Add subtle hover states (bg-muted on hover)

### Don't
- Don't use pure white (#fff) backgrounds
- Don't use sharp corners (always round)
- Don't use heavy shadows (keep it flat)
- Don't mix font families unnecessarily
- Don't animate everything (be subtle)

## File Organization

```
src/lib/components/
├── chat/           # Chat-specific components
│   ├── AsciiTitle.svelte
│   ├── PlasmaBackground.svelte
│   └── ...
├── common/         # Shared components
│   ├── Button.svelte
│   ├── Card.svelte
│   └── ...
├── landing/        # Marketing page components
│   ├── Hero.svelte
│   ├── Features.svelte
│   └── CrabMascot.svelte
└── ...
```
