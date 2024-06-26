export const SITE = {
  title: 'map2 docs',
  description: 'Linux key remapping tool map2 - official documentation',
  defaultLanguage: 'en-us'
} as const

export const OPEN_GRAPH = {
  image: {
    src: 'logo.png',
    alt: 'map2 logo - stylized letters "M" and "2"'
  },
}

export const KNOWN_LANGUAGES = {
  English: 'en'
} as const
export const KNOWN_LANGUAGE_CODES = Object.values(KNOWN_LANGUAGES)

export const EDIT_URL = `https://github.com/shiro/map2/docs`;

export const COMMUNITY_INVITE_URL = `https://discord.gg/brKgH43XQN`;
export const DONATE_URL = `https://ko-fi.com/shiroi_usagi`;

// See "Algolia" section of the README for more information.
export const ALGOLIA = {
  indexName: 'XXXXXXXXXX',
  appId: 'XXXXXXXXXX',
  apiKey: 'XXXXXXXXXX'
}

export type Sidebar = Record<
  (typeof KNOWN_LANGUAGE_CODES)[number],
  Record<string, { text: string; link: string }[]>
>
export const SIDEBAR: Sidebar = {
  en: {
    "Basics": [
      { text: "Introduction", link: "en/basics/introduction" },
      { text: "Install", link: "en/basics/install" },
      { text: "Getting started", link: "en/basics/getting-started" },
      { text: "Keys and key sequences", link: "en/basics/keys-and-key-sequences" },
      { text: "Routing", link: "en/basics/routing" },
    ],
    "Advanced": [
      { text: "Secure setup", link: "en/advanced/secure-setup" },
      { text: "Autostart", link: "en/advanced/autostart" },
    ],
    "API": [
      { text: "map2", link: "en/api/map2" },
      { text: "Reader", link: "en/api/reader" },
      { text: "Mapper", link: "en/api/mapper" },
      { text: "Text Mapper", link: "en/api/text-mapper" },
      { text: "Chord Mapper", link: "en/api/chord-mapper" },
      { text: "Writer", link: "en/api/writer" },
      { text: "Virtual Writer", link: "en/api/virtual-writer" },
      { text: "Window", link: "en/api/window" },
    ],
    "Examples": [
      { text: "Hello world", link: "en/examples/hello-world" },
      { text: "Chords", link: "en/examples/chords" },
      { text: "Text mapping", link: "en/examples/text-mapping" },
      { text: "WASD mouse control", link: "en/examples/wasd-mouse-control" },
      { text: "Keyboard to controller", link: "en/examples/keyboard-to-controller" },
    ]
  }
}
