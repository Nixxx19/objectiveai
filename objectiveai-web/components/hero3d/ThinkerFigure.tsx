"use client";

import { forwardRef, useRef, useMemo } from "react";
import * as THREE from "three";
import { RoundedBox } from "@react-three/drei";

interface ThinkerFigureProps {
  scale?: number;
  headTilt?: number;
  headNod?: number;
  position?: [number, number, number];
  rotation?: [number, number, number];
}

/**
 * Procedural 3D "Thinker" figure mascot for ObjectiveAI.
 * Built entirely from THREE.js primitives — no external models.
 *
 * The head group ref is forwarded so parent components can
 * drive nod/tilt animations externally.
 */
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
    const internalHeadRef = useRef<THREE.Group>(null);
    const resolvedHeadRef = (headGroupRef as React.RefObject<THREE.Group>) ?? internalHeadRef;

    // ---------------------------------------------------------------
    // Materials (memoised so they aren't recreated every render)
    // ---------------------------------------------------------------

    const innerMaterial = useMemo(
      () =>
        new THREE.MeshPhysicalMaterial({
          color: new THREE.Color("#E8E4F0"),
          roughness: 0.15,
          metalness: 0.0,
          clearcoat: 0.8,
          clearcoatRoughness: 0.1,
        }),
      []
    );

    const shellMaterial = useMemo(
      () =>
        new THREE.MeshPhysicalMaterial({
          color: new THREE.Color("#6B5CFF"),
          roughness: 0.1,
          metalness: 0.0,
          transmission: 0.85,
          thickness: 0.5,
          ior: 1.45,
          clearcoat: 1.0,
          clearcoatRoughness: 0.05,
          transparent: true,
          opacity: 0.4,
          side: THREE.DoubleSide,
          depthWrite: false,
        }),
      []
    );

    const eyeMaterial = useMemo(
      () =>
        new THREE.MeshStandardMaterial({
          color: new THREE.Color("#1B1B1B"),
          roughness: 0.3,
          metalness: 0.0,
        }),
      []
    );

    // ---------------------------------------------------------------
    // Leg geometry (tapered cylinder via CylinderGeometry)
    // ---------------------------------------------------------------

    const legGeometry = useMemo(
      () => new THREE.CylinderGeometry(0.05, 0.2, 1.0, 16),
      []
    );

    const neckGeometry = useMemo(
      () => new THREE.CylinderGeometry(0.3, 0.3, 0.3, 16),
      []
    );

    const eyeGeometry = useMemo(
      () => new THREE.SphereGeometry(0.12, 24, 24),
      []
    );

    // ---------------------------------------------------------------
    // Vertical layout (bottom-up, centred at origin, ~3 units tall)
    //
    //   Legs bottom:  -1.5
    //   Legs top:     -0.5
    //   Body centre:   0.0
    //   Body top:      0.5
    //   Neck centre:   0.65
    //   Neck top:      0.8
    //   Head centre:   1.5   (head is 1.4 tall, centre at 0.8 + 0.7)
    //   Shell centre:  1.5
    // ---------------------------------------------------------------

    const legsBottomY = -1.5;
    const legHeight = 1.0;
    const legCentreY = legsBottomY + legHeight / 2; // -1.0
    const legSpreadX = 0.3;

    const bodyCentreY = -0.5 + 0.5; // 0.0
    const bodyH = 1.0;

    const neckCentreY = bodyCentreY + bodyH / 2 + 0.15; // 0.65
    const neckTopY = neckCentreY + 0.15; // 0.8

    const headCentreY = neckTopY + 0.7; // 1.5

    return (
      <group
        position={position}
        rotation={rotation}
        scale={overallScale}
      >
        {/* ---- Legs ---- */}
        <mesh
          geometry={legGeometry}
          material={innerMaterial}
          position={[-legSpreadX, legCentreY, 0]}
        />
        <mesh
          geometry={legGeometry}
          material={innerMaterial}
          position={[legSpreadX, legCentreY, 0]}
        />

        {/* ---- Body ---- */}
        <RoundedBox
          args={[1.0, bodyH, 0.8]}
          radius={0.2}
          smoothness={8}
          position={[0, bodyCentreY, 0]}
        >
          <primitive object={innerMaterial} attach="material" />
        </RoundedBox>

        {/* ---- Neck ---- */}
        <mesh
          geometry={neckGeometry}
          material={innerMaterial}
          position={[0, neckCentreY, 0]}
        />

        {/* ---- Head group (independently rotatable) ---- */}
        <group
          ref={resolvedHeadRef}
          position={[0, headCentreY, 0]}
          rotation={[headNod, headTilt, 0]}
        >
          {/* Inner head */}
          <RoundedBox
            args={[1.2, 1.4, 1.0]}
            radius={0.25}
            smoothness={8}
          >
            <primitive object={innerMaterial} attach="material" />
          </RoundedBox>

          {/* Eyes — inset into the front face (positive Z) */}
          <mesh
            geometry={eyeGeometry}
            material={eyeMaterial}
            position={[-0.25, 0.1, 0.48]}
          />
          <mesh
            geometry={eyeGeometry}
            material={eyeMaterial}
            position={[0.25, 0.1, 0.48]}
          />

          {/* Shell — transparent acrylic envelope around head + upper neck */}
          <RoundedBox
            args={[1.4, 1.6, 1.2]}
            radius={0.3}
            smoothness={8}
          >
            <primitive object={shellMaterial} attach="material" />
          </RoundedBox>
        </group>

        {/* ---- Shell body extension (covers upper body, blends with head shell) ---- */}
        <RoundedBox
          args={[1.2, 0.7, 1.0]}
          radius={0.2}
          smoothness={8}
          position={[0, bodyCentreY + bodyH / 2 - 0.05, 0]}
        >
          <primitive object={shellMaterial} attach="material" />
        </RoundedBox>
      </group>
    );
  }
);

ThinkerFigure.displayName = "ThinkerFigure";

export default ThinkerFigure;
