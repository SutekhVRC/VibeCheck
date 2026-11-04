/** A theme colour role, read from the --theme-<name> CSS variable */
const role = (name) => `rgb(var(--theme-${name}) / <alpha-value>)`;

/** @type {import('tailwindcss').Config} */
module.exports = {
	darkMode: ["class"],
	content: [
		'./pages/**/*.{ts,tsx}',
		'./components/**/*.{ts,tsx}',
		'./app/**/*.{ts,tsx}',
		'./src/**/*.{ts,tsx}',
	],
	theme: {
		container: {
			center: true,
			padding: '2rem',
			screens: {
				'2xl': '1400px'
			}
		},
		extend: {
			keyframes: {
				'accordion-down': {
					from: {
						height: 0
					},
					to: {
						height: 'var(--radix-accordion-content-height)'
					}
				},
				'accordion-up': {
					from: {
						height: 'var(--radix-accordion-content-height)'
					},
					to: {
						height: 0
					}
				},
				wiggle: {
					'0%, 100%': { transform: 'rotate(-8deg)' },
					'50%': { transform: 'rotate(8deg)' }
				},
				float: {
					'0%, 100%': { transform: 'translateY(0)' },
					'50%': { transform: 'translateY(-4px)' }
				}
			},
			animation: {
				'accordion-down': 'accordion-down 0.2s ease-out',
				'accordion-up': 'accordion-up 0.2s ease-out',
				wiggle: 'wiggle 0.5s ease-in-out infinite',
				float: 'float 3s ease-in-out infinite'
			},
			borderRadius: {
				lg: 'var(--radius)',
				md: 'calc(var(--radius) - 2px)',
				sm: 'calc(var(--radius) - 4px)'
			},
			// Every colour comes from the active theme (src/theme/), which sets
			// --theme-<role> CSS variables as "r g b" so opacity modifiers like
			// bg-theme-primary/20 keep working.
			colors: {
				theme: {
					background: role('background'),
					surface: role('surface'),
					raised: role('raised'),
					border: role('border'),
					text: role('text'),
					muted: role('muted'),
					primary: role('primary'),
					secondary: role('secondary'),
					success: role('success'),
					danger: role('danger'),
					info: role('info'),
					warning: role('warning'),
					'on-accent': role('on-accent')
				},
				// shadcn/ui component tokens, mapped onto the theme roles
				background: role('background'),
				foreground: role('text'),
				card: { DEFAULT: role('surface'), foreground: role('text') },
				popover: { DEFAULT: role('surface'), foreground: role('text') },
				primary: { DEFAULT: role('primary'), foreground: role('on-accent') },
				secondary: { DEFAULT: role('raised'), foreground: role('text') },
				muted: { DEFAULT: role('raised'), foreground: role('muted') },
				accent: { DEFAULT: role('secondary'), foreground: role('on-accent') },
				destructive: { DEFAULT: role('danger'), foreground: role('on-accent') },
				border: role('border'),
				input: role('border'),
				ring: role('secondary'),
				chart: {
					'1': role('primary'),
					'2': role('secondary'),
					'3': role('success'),
					'4': role('info'),
					'5': role('warning')
				}
			},
			// Theme shadows: the active theme's shadow style (glow / neon / hard /
			// none) sets these variables, see src/theme/styles.ts
			boxShadow: {
				'theme-panel': 'var(--theme-shadow-panel)',
				'theme-button': 'var(--theme-shadow-button)',
				'theme-pop': 'var(--theme-shadow-pop)',
				'theme-tooltip': 'var(--theme-shadow-tooltip)'
			},
			dropShadow: {
				'theme-glow': 'var(--theme-shadow-drop)'
			}
		}
	},
	plugins: [require("tailwindcss-animate")],
}