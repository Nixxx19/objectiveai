package objectiveai

import (
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

// OrderedMap is a JSON-order-preserving map. It wraps orderedmap.OrderedMap
// in a value-type struct so that:
//   - Value fields marshal correctly (no pointer-receiver issue)
//   - *OrderedMap[K, V] expresses nullability naturally
type OrderedMap[K comparable, V any] struct {
	inner *orderedmap.OrderedMap[K, V]
}

func NewOrderedMap[K comparable, V any](pairs ...orderedmap.Pair[K, V]) OrderedMap[K, V] {
	return OrderedMap[K, V]{inner: orderedmap.New[K, V](orderedmap.WithInitialData(pairs...))}
}

func (om OrderedMap[K, V]) Set(key K, value V) {
	if om.inner == nil {
		return
	}
	om.inner.Set(key, value)
}

func (om OrderedMap[K, V]) Get(key K) (V, bool) {
	if om.inner == nil {
		var zero V
		return zero, false
	}
	return om.inner.Get(key)
}

func (om OrderedMap[K, V]) Len() int {
	if om.inner == nil {
		return 0
	}
	return om.inner.Len()
}

func (om OrderedMap[K, V]) Oldest() *orderedmap.Pair[K, V] {
	if om.inner == nil {
		return nil
	}
	return om.inner.Oldest()
}

func (om OrderedMap[K, V]) Newest() *orderedmap.Pair[K, V] {
	if om.inner == nil {
		return nil
	}
	return om.inner.Newest()
}

func (om OrderedMap[K, V]) Delete(key K) (V, bool) {
	if om.inner == nil {
		var zero V
		return zero, false
	}
	return om.inner.Delete(key)
}

func (om OrderedMap[K, V]) MarshalJSON() ([]byte, error) {
	if om.inner == nil {
		return []byte("{}"), nil
	}
	return om.inner.MarshalJSON()
}

func (om *OrderedMap[K, V]) UnmarshalJSON(data []byte) error {
	if om.inner == nil {
		om.inner = orderedmap.New[K, V]()
	}
	return om.inner.UnmarshalJSON(data)
}
