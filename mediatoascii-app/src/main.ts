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
            400: '#f47ab8',
            500: '#d94a8c',
            600: '#c23078',
            700: '#a02060',
            800: '#801050',
            900: '#600840',
            950: '#400528'
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
