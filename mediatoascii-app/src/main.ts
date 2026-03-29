import { createApp } from "vue";
import App from "./App.vue";
import PrimeVue from 'primevue/config';
import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';
import './style.css'

const MyPreset = definePreset(Aura, {
    semantic: {
        primary: {
            50: '#fff5f8',
            100: '#ffe6f2',
            200: '#ffd6eb',
            300: '#ffb8db',
            400: '#ffb8db',
            500: '#ffb8db',
            600: '#ffb8db',
            700: '#ffb8db',
            800: '#fcafd4',
            900: '#fda4d1',
            950: '#ff92c6'
        },
        colorScheme: {
            light: {
                primary: {
                    color: '{primary.500}',
                    contrastColor: '#ffffff',
                    hoverColor: '{primary.600}',
                    activeColor: '{primary.700}'
                }
            }
        }
    }
});

createApp(App).use(PrimeVue, {
    theme: {
        preset: MyPreset
    }
}).mount("#app");
