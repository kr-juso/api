package csv

import (
	"bytes"
	_ "embed"
	"encoding/csv"

	"github.com/kr-juso/api/internal/wildcard"
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
		if row[2] == "존재" {
			regcodes[i] = model.Regcode{
				Code: row[0],
				Name: row[1],
			}
		}
	}
}

// codePattern이 00000000* 패턴일 경우 strings.HasPrefix 체크
// codePattern이 *00000000 패턴일 경우 strings.HasSuffix 체크
func GetRegcodes(codePattern string) []model.Regcode {
	result := make([]model.Regcode, 0)

	for _, regcode := range regcodes {
		if wildcard.IsMatch(regcode.Code, codePattern) {
			result = append(result, regcode)
		}
	}

	return result
}
