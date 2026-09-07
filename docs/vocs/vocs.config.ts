import { defineConfig } from 'vocs/config'
import { sidebar } from './sidebar'
import { basePath } from './redirects.config'

export default defineConfig({
  title: 'CreditChain',
  description: 'CreditChain is a Rust-first, Reth-based, EVM-compatible public chain for AI-native financial infrastructure.',
  logoUrl: '/logo.png',
  iconUrl: '/logo.png',
  ogImageUrl: '/logo.png',
  accentColor: 'light-dark(#1f1f1f, #ffffff)',
  srcDir: 'docs',
  outDir: 'docs/dist',
  renderStrategy: 'full-static',
  sidebar,
  basePath,
  search: {
    fuzzy: true
  },
  topNav: [
    { text: 'Run', link: '/run/ethereum' },
    { text: 'SDK', link: '/sdk' },
    {
      element: React.createElement('a', { href: '/docs', target: '_self' }, 'Rustdocs')
    },
    { text: 'GitHub', link: 'https://github.com/openibank/creditchain' },
    {
      text: 'v2.5.2',
      items: [
        {
          text: 'Releases',
          link: 'https://github.com/openibank/creditchain/releases'
        },
        {
          text: 'Contributing',
          link: 'https://github.com/openibank/creditchain/blob/main/CONTRIBUTING.md'
        }
      ]
    }
  ],
  socials: [
    {
      icon: 'github',
      link: 'https://github.com/openibank/creditchain',
    },
    {
      icon: 'telegram',
      link: 'https://docs.creditchain.org',
    },
  ],
  sponsors: [
    {
      name: 'Collaborators',
      height: 120,
      items: [
        [
          {
            name: 'OpeniBank',
            link: 'https://www.openibank.com',
            image: '/logo.png',
          },
          {
            name: 'CreditChain',
            link: 'https://www.creditchain.org',
            image: '/logo.png',
          }
        ]
      ]
    }
  ],
  theme: {
    accentColor: {
      light: '#1f1f1f',
      dark: '#ffffff',
    }
  },
  editLink: {
    pattern: "https://github.com/openibank/creditchain/edit/main/docs/vocs/docs/pages/:path",
  },
  vite: {
    plugins: [
      {
        name: 'transform-summary-links',
        apply: 'serve', // only during dev for faster feedback
        enforce: 'pre',
        async load(id) {
          if (id.endsWith('pages/cli/SUMMARY.mdx') || id.endsWith('pages/cli/summary.mdx')) {
            const { readFileSync } = await import('node:fs')
            let code = readFileSync(id, 'utf-8')
            code = code.replace(/\]\(\.\/([^)]+)\.mdx\)/g, '](/cli/\$1)')
            return code
          }
        }
      },
      {
        name: 'transform-summary-links-build',
        apply: 'build', // only apply during build
        enforce: 'pre',
        async load(id) {
          if (id.endsWith('pages/cli/SUMMARY.mdx') || id.endsWith('pages/cli/summary.mdx')) {
            const { readFileSync } = await import('node:fs')
            let code = readFileSync(id, 'utf-8')
            code = code.replace(/\]\(\.\/([^)]+)\.mdx\)/g, '](/cli/\$1)')
            return code
          }
        }
      }
    ]
  }
})
