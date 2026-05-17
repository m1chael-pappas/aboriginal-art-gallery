<script setup lang="ts">
import { onBeforeUnmount, onMounted, nextTick, ref, watch } from 'vue'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'

// Renders a tribe's territory (a GeoJSON geometry, typically a MultiPolygon)
// on a read-only Leaflet map. The geometry comes straight from the API's
// ST_AsGeoJSON output, so this is the same data the raw-GeoJSON panel shows,
// just drawn instead of dumped.
const props = defineProps<{ geometry: unknown | null }>()

// L.geoJSON's first argument type, reused so we can cast the loosely-typed
// prop without resorting to `any`.
type GeoInput = Parameters<typeof L.geoJSON>[0]

const el = ref<HTMLDivElement | null>(null)
let map: L.Map | null = null
let layer: L.GeoJSON | null = null

// Australia, roughly - the view before (or instead of) a fitted polygon.
const AUS_CENTER: L.LatLngExpression = [-25.6, 134.4]

function draw() {
  if (!map) return
  if (layer) {
    layer.remove()
    layer = null
  }
  if (!props.geometry) {
    map.setView(AUS_CENTER, 4)
    return
  }
  try {
    layer = L.geoJSON(props.geometry as GeoInput, {
      // Ochre to match the catalog/archive palette (see style.css @theme).
      style: { color: '#b5651d', weight: 2, fillColor: '#b5651d', fillOpacity: 0.15 },
    }).addTo(map)
    const bounds = layer.getBounds()
    if (bounds.isValid()) {
      map.fitBounds(bounds, { padding: [24, 24], maxZoom: 9 })
    } else {
      map.setView(AUS_CENTER, 4)
    }
  } catch {
    // Malformed geometry: leave the basemap visible rather than crash the
    // detail page. The raw-GeoJSON panel still lets the user inspect it.
    map.setView(AUS_CENTER, 4)
  }
}

onMounted(async () => {
  if (!el.value) return
  map = L.map(el.value, {
    // Don't hijack page scroll; the user can still zoom with the +/- control.
    scrollWheelZoom: false,
  })
  L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png', {
    attribution:
      '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>',
    maxZoom: 19,
  }).addTo(map)
  map.setView(AUS_CENTER, 4)
  // The container is sized by CSS; Leaflet must re-measure after the DOM
  // settles or it renders into a 0×0 box.
  await nextTick()
  map.invalidateSize()
  draw()
})

watch(
  () => props.geometry,
  () => draw(),
)

onBeforeUnmount(() => {
  map?.remove()
  map = null
  layer = null
})
</script>

<template>
  <div ref="el" class="h-80 w-full border border-line bg-paper"></div>
</template>
