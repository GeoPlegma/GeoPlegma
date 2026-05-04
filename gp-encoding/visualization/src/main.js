import {Deck, WebMercatorViewport, _GlobeView as GlobeView} from '@deck.gl/core';
import {H3HexagonLayer} from '@deck.gl/geo-layers';
import {GeoJsonLayer} from '@deck.gl/layers';
import {cellToBoundary} from 'h3-js';

const BAND_KEY_PREFIX = 'band_';

function getBandCount(sample) {
  if (!sample || typeof sample !== 'object') {
    return 0;
  }

  return Object.keys(sample).filter(key => key.startsWith(BAND_KEY_PREFIX)).length;
}

const H3_CELLS_URL = '/h3cells.json';
const COUNTRIES_BORDERS_URL = '/countries.geojson';
const VIEW_PADDING = 64;

function getBoundsFromH3Cells(cells) {
  const bounds = {
    west: Number.POSITIVE_INFINITY,
    south: Number.POSITIVE_INFINITY,
    east: Number.NEGATIVE_INFINITY,
    north: Number.NEGATIVE_INFINITY
  };

  for (const cell of cells) {
    const boundary = cellToBoundary(cell.hex, true);

    for (const [longitude, latitude] of boundary) {
      bounds.west = Math.min(bounds.west, longitude);
      bounds.south = Math.min(bounds.south, latitude);
      bounds.east = Math.max(bounds.east, longitude);
      bounds.north = Math.max(bounds.north, latitude);
    }
  }

  if (!Number.isFinite(bounds.west)) {
    return [[-180, -90], [180, 90]];
  }

  return [
    [bounds.west, bounds.south],
    [bounds.east, bounds.north]
  ];
}

function getInitialViewState(bounds) {
  const width = appContainer?.clientWidth || window.innerWidth;
  const height = appContainer?.clientHeight || window.innerHeight;
  const viewport = new WebMercatorViewport({width, height});
  const {longitude, latitude, zoom} = viewport.fitBounds(bounds, {
    padding: VIEW_PADDING,
    maxZoom: 20
  });

  return {
    longitude,
    latitude,
    zoom,
    maxZoom: 20,
    pitch: 30,
    bearing: 0
  };
}


function createH3Layer(cells) {
  const bandCount = getBandCount(cells?.[0]);
  const useElevation = bandCount === 1;

  return new H3HexagonLayer({
    id: 'H3HexagonLayer',
    data: cells,
    elevationScale: 20,
    extruded: useElevation,
    filled: true,
    getElevation: useElevation ? d => d.band_0 / 2 : 0,
    getFillColor: useElevation
      ? d => [d.band_0, d.band_0, d.band_0]
      : d => [d.band_0, d.band_1, d.band_2],
    getHexagon: d => d.hex,
    wireframe: false,
    pickable: true,
  });
}

const countriesBordersLayer = new GeoJsonLayer({
  id: 'CountriesBordersLayer',
  data: COUNTRIES_BORDERS_URL,
  stroked: true,
  filled: false,
  lineWidthMinPixels: 1,
  getLineColor: [120, 132, 145, 120],
  pickable: false
});

const appContainer = document.getElementById('app');

if (appContainer) {
  appContainer.addEventListener('contextmenu', event => {
    event.preventDefault();
  });
}

async function bootstrap() {
  const cells = await fetch(H3_CELLS_URL).then(response => response.json());
  const initialViewState = getInitialViewState(getBoundsFromH3Cells(cells));

  const h3Layer = createH3Layer(cells);

  new Deck({
    parent: appContainer || document.body,
    views: new GlobeView({id: 'globe'}),
    mapStyle: 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json',
    initialViewState,
    controller: true,
    getTooltip: ({object}) => object && `${object.hex}`,
    layers: [
      countriesBordersLayer,
      h3Layer
    ]
  });
}

bootstrap().catch(error => {
  console.error('Failed to initialise visualization', error);
});
  