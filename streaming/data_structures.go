package streaming

import (
	"io"

	"github.com/dgraph-io/badger/v2"
	"github.com/golang/protobuf/proto"
	"github.com/pkg/errors"
)

var ErrEndOfIterator = errors.New("end of iterator")

type Iterator interface {
	Next(proto.Message) error
	io.Closer
}

/* LinkedList */
type LinkedList struct {
	tx           *badgerTransaction
	elementCount int
}

func NewLinkedList(tx *badgerTransaction) *LinkedList {
	return &LinkedList{
		tx:           tx,
		elementCount: 0,
	}
}

func (ll *LinkedList) Append(value proto.Message) error {
	data, err := proto.Marshal(value)
	if err != nil {
		return errors.Wrap(err, "couldn't serialize given value")
	}

	err = ll.tx.Set(intToByteSlice(ll.elementCount), data)
	if err != nil {
		return errors.Wrap(err, "couldn't add the element to linked list")
	}

	ll.elementCount += 1
	return nil
}

func (ll *LinkedList) GetAll() (*LinkedListIterator, error) {
	options := badger.DefaultIteratorOptions

	it := ll.tx.tx.NewIterator(options)

	return newLinkedListIterator(it), nil
}

type LinkedListIterator struct {
	it     *badger.Iterator
	closed bool
}

func newLinkedListIterator(it *badger.Iterator) *LinkedListIterator {
	return &LinkedListIterator{
		it: it,
	}
}

func (lli *LinkedListIterator) Next(value proto.Message) error {
	if !lli.it.Valid() {
		return ErrEndOfIterator
	}

	lli.it.Next()
	item := lli.it.Item()

	err := item.Value(func(val []byte) error {
		err := proto.Unmarshal(val, value)
		return err
	})

	if err != nil {
		return errors.Wrap(err, "couldn't unmarshal data")
	}

	return nil
}

func (lli *LinkedListIterator) Close() {
	lli.it.Close()
}

func intToByteSlice(x int) []byte {
	return []byte(string(x))
}

/* Map */
type Map struct {
	tx *badgerTransaction
}

func NewMap(tx *badgerTransaction) *Map {
	return &Map{
		tx: tx,
	}
}

func (hm *Map) Set(key []byte, value proto.Message) error {
	data, err := proto.Marshal(value)
	if err != nil {
		return errors.Wrap(err, "couldn't marshal value")
	}

	err = hm.tx.Set(key, data)
	if err != nil {
		return errors.Wrap(err, "couldn't add element to dictionary")
	}

	return nil
}

func (hm *Map) Get(key []byte, value proto.Message) error {
	data, err := hm.tx.Get(key)
	if err != nil {
		return errors.Wrap(err, "couldn't get element from dictionary")
	}

	err = proto.Unmarshal(data, value)
	return err
}

func (hm *Map) GetAllWithPrefix(prefix []byte) *MapIterator {
	options := badger.DefaultIteratorOptions
	options.Prefix = prefix

	it := hm.tx.tx.NewIterator(options)

	return newMapIterator(it)
}

func (hm *Map) GetAll() *MapIterator {
	options := badger.DefaultIteratorOptions

	it := hm.tx.tx.NewIterator(options)

	return newMapIterator(it)
}

type MapIterator struct {
	it *badger.Iterator
}

func newMapIterator(it *badger.Iterator) *MapIterator {
	return &MapIterator{
		it: it,
	}
}

func (mi *MapIterator) Next(value proto.Message) error {
	if !mi.it.Valid() {
		return ErrEndOfIterator
	}

	mi.it.Next()
	item := mi.it.Item()

	err := item.Value(func(val []byte) error {
		err := proto.Unmarshal(val, value)
		return err
	})

	if err != nil {
		return errors.Wrap(err, "couldn't unmarshal data")
	}

	return nil
}

func (mi *MapIterator) Close() {
	mi.it.Close()
}
