import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { applyDisplayPrefs } from './lib/prefs'

applyDisplayPrefs()

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
