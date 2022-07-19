package csv

import (
	"bytes"
	_ "embed"
	"encoding/csv"

	"strings"

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
func GetRegcodes(codePattern string, isIgnoreZero bool) []model.Regcode {
	result := make([]model.Regcode, 0)

	for _, regcode := range regcodes {
		isOk := wildcard.IsMatch(regcode.Code, codePattern)
		if isIgnoreZero {
			isOk = isOk && !isStarStartsWithZero(regcode.Code, codePattern)
		}

		if isOk {
			result = append(result, regcode)
		}
	}

	return result
}

func isStarStartsWithZero(regcode, codePattern string) bool {
	starPosition := strings.Index(codePattern, "*")
	if starPosition == -1 {
		return false
	}

	return regcode[starPosition:min(len(regcode), starPosition+2)] == "00"
}

func min(a, b int) int {
	if a < b {
		return a
	}

	return b
}
