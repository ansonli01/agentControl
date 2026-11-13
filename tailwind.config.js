/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        base: {
          900: "#FFFFFF",
          850: "#FAFAFB",
          800: "#FFFFFF",
          700: "#F3F4F6",
          600: "#E6E8EC",
        },
        ink: {
          DEFAULT: "#1D2129",
          soft: "#4E5969",
          mute: "#86909C",
        },
        brand: {
          DEFAULT: "#3B6EF5",
          strong: "#2A57D4",
          light: "#6A93FF",
        },
        ok: "#00B42A",
        warn: "#FF7D00",
        err: "#F53F3F",
        magic: "#7B5BF0",
      },
      fontFamily: {
        sans: [
          "PingFang SC",
          "Microsoft YaHei",
          "Segoe UI",
          "system-ui",
          "sans-serif",
        ],
        mono: ["Cascadia Mono", "Consolas", "JetBrains Mono", "monospace"],
      },
      fontSize: {
        h1: ["22px", { lineHeight: "30px", fontWeight: "600" }],
        h2: ["16px", { lineHeight: "24px", fontWeight: "500" }],
        body: ["14px", { lineHeight: "22px", fontWeight: "400" }],
      },
      borderRadius: {
        card: "12px",
        pill: "999px",
      },
      boxShadow: {
        card: "0 2px 8px -2px rgba(29, 33, 41, 0.08)",
        lift: "0 10px 28px -10px rgba(29, 33, 41, 0.16)",
      },
      keyframes: {
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
        pulseRing: {
          "0%, 100%": { opacity: "0.35" },
          "50%": { opacity: "1" },
        },
      },
      animation: {
        shimmer: "shimmer 1.2s linear infinite",
        "pulse-ring": "pulseRing 1.4s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};