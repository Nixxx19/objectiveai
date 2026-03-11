"use client";

import { forwardRef, useRef, useMemo } from "react";
import * as THREE from "three";
import { useGLTF } from "@react-three/drei";

interface ThinkerFigureProps {
  scale?: number;
  headTilt?: number;
  headNod?: number;
  position?: [number, number, number];
  rotation?: [number, number, number];
}

// Brand-matched materials (glTF PBR often looks too dark without env maps)
function createOverrideMaterials() {
  return {
    Body_Vinyl: new THREE.MeshPhysicalMaterial({
      color: new THREE.Color("#E8E4F0"),
      roughness: 0.35,
      metalness: 0,
      clearcoat: 0.6,
      clearcoatRoughness: 0.15,
    }),
    Head_Inner: new THREE.MeshPhysicalMaterial({
      color: new THREE.Color("#F0EDF5"),
      roughness: 0.3,
      metalness: 0,
      clearcoat: 0.8,
      clearcoatRoughness: 0.1,
    }),
    Helmet_Glass: new THREE.MeshPhysicalMaterial({
      color: new THREE.Color("#6B5CFF"),
      roughness: 0.08,
      metalness: 0,
      transmission: 0.85,
      thickness: 0.4,
      ior: 1.45,
      clearcoat: 1.0,
      clearcoatRoughness: 0.05,
      transparent: true,
      opacity: 0.45,
      side: THREE.DoubleSide,
      depthWrite: false,
    }),
    Eye_Black: new THREE.MeshPhysicalMaterial({
      color: new THREE.Color("#1B1B1B"),
      roughness: 0.3,
      metalness: 0,
    }),
  };
}

const ThinkerFigure = forwardRef<THREE.Group, ThinkerFigureProps>(
  (
    {
      scale: overallScale = 1,
      headTilt = 0,
      headNod = 0,
      position = [0, 0, 0],
      rotation = [0, 0, 0],
    },
    headGroupRef
  ) => {
    const { scene } = useGLTF("/hero/thinker.glb");
    const internalHeadRef = useRef<THREE.Group>(null);
    const resolvedHeadRef =
      (headGroupRef as React.RefObject<THREE.Group>) ?? internalHeadRef;

    // Clone scene + override materials synchronously before render
    const clone = useMemo(() => {
      const c = scene.clone(true);
      const mats = createOverrideMaterials();
      c.traverse((child) => {
        if (child instanceof THREE.Mesh && child.material) {
          const name = (child.material as THREE.Material).name;
          if (name in mats) {
            child.material = mats[name as keyof typeof mats];
          }
        }
      });
      return c;
    }, [scene]);

    // Model bbox: Y [0.7, 2.3] → center at 1.5, height 1.6
    // Old procedural figure: Y [-1.5, 1.5] → center at 0, height 3.0
    const modelScale = 1.875;
    const modelOffsetY = -1.5;

    return (
      <group position={position} rotation={rotation} scale={overallScale}>
        <group ref={resolvedHeadRef} rotation={[headNod, headTilt, 0]}>
          <group scale={modelScale} position={[0, modelOffsetY, 0]}>
            <primitive object={clone} />
          </group>
        </group>
      </group>
    );
  }
);

ThinkerFigure.displayName = "ThinkerFigure";

export default ThinkerFigure;
