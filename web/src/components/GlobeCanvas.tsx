import { useEffect, useRef } from 'react';
import {
  AdditiveBlending,
  AmbientLight,
  BackSide,
  Color,
  DirectionalLight,
  DynamicDrawUsage,
  InstancedMesh,
  Matrix4,
  Mesh,
  MeshPhongMaterial,
  PerspectiveCamera,
  Raycaster,
  Scene,
  SphereGeometry,
  Vector2,
  WebGLRenderer
} from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { OrbitalObject } from '../types/orbit';

const EARTH_RADIUS_KM = 6378.137;
const SCALE_FACTOR = 1 / EARTH_RADIUS_KM;
const MAX_OBJECTS = 10_000;

export interface GlobeCanvasProps {
  objects: OrbitalObject[];
  onSelect?: (object: OrbitalObject) => void;
  highlightedIds?: Set<string>;
}

export function GlobeCanvas({ objects, onSelect, highlightedIds }: GlobeCanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const instancedRef = useRef<InstancedMesh | null>(null);
  const objectsRef = useRef<OrbitalObject[]>([]);

  useEffect(() => {
    const container = containerRef.current;
    const canvas = canvasRef.current;
    if (!container || !canvas) {
      return;
    }

    const renderer = new WebGLRenderer({ canvas, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(container.clientWidth, container.clientHeight);

    const scene = new Scene();
    scene.background = new Color('#020409');

    const camera = new PerspectiveCamera(45, container.clientWidth / container.clientHeight, 0.1, 50);
    camera.position.set(0, 0, 6);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.minDistance = 1.2;
    controls.maxDistance = 10;

    const earthGeometry = new SphereGeometry(1, 64, 64);
    const earthMaterial = new MeshPhongMaterial({
      color: new Color('#123f9c'),
      emissive: new Color('#05122c'),
      shininess: 18,
      specular: new Color('#2b6fff')
    });
    const earth = new Mesh(earthGeometry, earthMaterial);
    scene.add(earth);

    const atmosphereMaterial = new MeshPhongMaterial({
      color: new Color('#4c87ff'),
      transparent: true,
      blending: AdditiveBlending,
      opacity: 0.12,
      side: BackSide
    });
    const atmosphere = new Mesh(earthGeometry.clone().scale(1.04, 1.04, 1.04), atmosphereMaterial);
    scene.add(atmosphere);

    const ambient = new AmbientLight(0x0f1830, 0.9);
    scene.add(ambient);
    const daylight = new DirectionalLight(0xffffff, 1.15);
    daylight.position.set(-2, 1.5, 1.5);
    scene.add(daylight);

    const satelliteGeometry = new SphereGeometry(0.015, 12, 12);
    const satelliteMaterial = new MeshPhongMaterial({ color: 0xffffff });
    const instanced = new InstancedMesh(satelliteGeometry, satelliteMaterial, MAX_OBJECTS);
    instanced.instanceMatrix.setUsage(DynamicDrawUsage);
    instanced.instanceMatrix.needsUpdate = true;
    scene.add(instanced);
    instancedRef.current = instanced;

    const raycaster = new Raycaster();
    const pointer = new Vector2();

    const onPointerDown = (event: MouseEvent) => {
      if (!instancedRef.current || !container) {
        return;
      }
      const bounds = container.getBoundingClientRect();
      pointer.x = ((event.clientX - bounds.left) / bounds.width) * 2 - 1;
      pointer.y = -((event.clientY - bounds.top) / bounds.height) * 2 + 1;
      raycaster.setFromCamera(pointer, camera);
      const intersections = raycaster.intersectObject(instancedRef.current);
      if (intersections.length > 0) {
        const instanceId = intersections[0].instanceId ?? undefined;
        if (instanceId !== undefined) {
          const selected = objectsRef.current[instanceId];
          if (selected && onSelect) {
            onSelect(selected);
          }
        }
      }
    };

    container.addEventListener('pointerdown', onPointerDown);

    const resizeObserver = new ResizeObserver(() => {
      if (!container) {
        return;
      }
      const { clientWidth, clientHeight } = container;
      renderer.setSize(clientWidth, clientHeight);
      camera.aspect = clientWidth / clientHeight;
      camera.updateProjectionMatrix();
    });

    resizeObserver.observe(container);

    let animationFrame = 0;
    const animate = () => {
      controls.update();
      renderer.render(scene, camera);
      animationFrame = requestAnimationFrame(animate);
    };
    animate();

    return () => {
      cancelAnimationFrame(animationFrame);
      resizeObserver.disconnect();
      container.removeEventListener('pointerdown', onPointerDown);
      instancedRef.current = null;
      renderer.dispose();
      satelliteGeometry.dispose();
      satelliteMaterial.dispose();
      earthGeometry.dispose();
      earthMaterial.dispose();
      atmosphereMaterial.dispose();
    };
  }, [onSelect]);

  useEffect(() => {
    objectsRef.current = objects.slice(0, MAX_OBJECTS);
    const instanced = instancedRef.current;
    if (!instanced) {
      return;
    }
    const matrix = new Matrix4();
    const color = new Color();
    const highlightSet = highlightedIds ?? new Set<string>();

    const length = Math.min(objectsRef.current.length, MAX_OBJECTS);
    instanced.count = length;

    for (let i = 0; i < length; i += 1) {
      const object = objectsRef.current[i];
      const [x, y, z] = object.position;
      matrix.setPosition(x * SCALE_FACTOR, y * SCALE_FACTOR, z * SCALE_FACTOR);
      instanced.setMatrixAt(i, matrix);
      color.set(getObjectColor(object, highlightSet.has(object.id)));
      instanced.setColorAt(i, color);
    }

    instanced.instanceMatrix.needsUpdate = true;
    if (instanced.instanceColor) {
      instanced.instanceColor.needsUpdate = true;
    }
  }, [objects, highlightedIds]);

  return (
    <div ref={containerRef} className="globe-container">
      <canvas ref={canvasRef} />
      <div className="globe-overlay" />
    </div>
  );
}

function getObjectColor(object: OrbitalObject, isHighlighted: boolean): Color {
  if (isHighlighted) {
    return new Color('#ff5b7d');
  }
  switch (object.health.status) {
    case 'critical':
      return new Color('#ff5b7d');
    case 'warning':
      return new Color('#f7a047');
    default:
      return object.kind === 'debris' ? new Color('#8fb4ff') : new Color('#3af2c3');
  }
}
