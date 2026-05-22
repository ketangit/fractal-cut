'use client';

import { useMemo } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { SVGLoader } from 'three/examples/jsm/loaders/SVGLoader.js';
import * as THREE from 'three';

interface Props {
  svgString: string;
  thicknessMm?: number;
}

function PuzzleMesh({ svgString, thicknessMm = 6 }: Props) {
  const geometry = useMemo(() => {
    const loader = new SVGLoader();
    const data = loader.parse(svgString);
    const shapes = data.paths.flatMap((p) => SVGLoader.createShapes(p));
    if (!shapes.length) return null;
    const merged = new THREE.Group();
    shapes.forEach((shape) => {
      const geo = new THREE.ExtrudeGeometry(shape, {
        depth: thicknessMm,
        bevelEnabled: false,
      });
      merged.add(new THREE.Mesh(geo));
    });
    return merged;
  }, [svgString, thicknessMm]);

  if (!geometry) return null;

  return (
    // Flip Y-axis (SVG → Three.js coordinate system) and scale SVG units → scene units
    <group rotation={[Math.PI, 0, 0]} scale={0.1}>
      {geometry.children.map((child, i) => (
        <mesh key={i} geometry={(child as THREE.Mesh).geometry}>
          <meshStandardMaterial color="#d4a96a" side={THREE.DoubleSide} />
        </mesh>
      ))}
    </group>
  );
}

export function PuzzleScene({ svgString, thicknessMm = 6 }: Props) {
  return (
    <div className="w-full h-96 bg-slate-900 rounded overflow-hidden">
      <Canvas camera={{ position: [0, 0, 80], fov: 50 }}>
        <ambientLight intensity={0.4} />
        <directionalLight position={[10, 20, 10]} intensity={1.2} />
        <PuzzleMesh svgString={svgString} thicknessMm={thicknessMm} />
        <OrbitControls enableDamping />
      </Canvas>
    </div>
  );
}
