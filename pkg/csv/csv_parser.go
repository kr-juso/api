package csv

import (
	"bytes"
	_ "embed"
	"encoding/csv"

	"github.com/kr-juso/api/pkg/model"
)

//go:embed internal/address.csv
var address []byte
var regcodes []model.Regcode

func init() {
	r := csv.NewReader(
		bytes.NewReader(address),
	)

	r.Comma = '\t'

	rows, err := r.ReadAll()
	if err != nil {
		panic(err)
	}

	regcodes = make([]model.Regcode, len(rows))

	for i, row := range rows {
		regcodes[i] = model.Regcode{
			Code: row[0],
			Name: row[1],
		}
	}
}
