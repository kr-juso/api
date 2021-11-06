package service

import (
	"context"
	pb "github.com/kr-juso/api/internal/grpc/juso/regcode"
	"github.com/kr-juso/api/pkg/csv"
)

type RegcodeService struct {
	pb.UnimplementedRegcodeServiceServer
}

func (service *RegcodeService) ListRegcodes(ctx context.Context, req *pb.ListRegcodesRequest) (*pb.ListRegcodesResponse, error) {
	regCodes := csv.GetRegcodes(
		req.RegcodePattern,
		req.IsIgnoreZero,
	)

	result := make([]*pb.Regcode, 0)

	for _, item := range regCodes {
		regCode := &pb.Regcode{
			Code: item.Code,
			Name: item.Name,
		}

		result = append(result, regCode)
	}

	response := &pb.ListRegcodesResponse{
		Regcodes: result,
	}

	return response, nil
}
